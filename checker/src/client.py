import asyncio
import logging
from logging import LoggerAdapter
from typing import Callable, Any, Literal

from enochecker3 import MumbleException
from httpx import AsyncClient, Response


class JitterishClient:
    def __init__(self, logger: LoggerAdapter, client: AsyncClient) -> None:
        self.logger = logger
        self.client = client

    def _assert_status_code(self, r: Response, code: int = 200, parse: Callable[[str], str] | None = None) -> None:
        if r.status_code == code:
            return
        errlog = r.text
        if parse is not None:
            errlog = parse(errlog)
        self.logger.error(f"Bad status code during {r.request.method} {r.request.url.path}: " \
                          + f"({r.status_code} != {code})\n{errlog}")
        raise MumbleException(f"{r.request.method} {r.request.url.path} failed")

    async def get(self, url: str, code: int = 200, **kwargs: Any) -> Response:
        response = await self.client.get(url, **kwargs)
        # self.logger.info(f'{response.request.method} {url} => {response.status_code}')
        self._assert_status_code(response, code)
        return response

    async def post(self, url: str, code: int = 200, **kwargs: Any) -> Response:
        response = await self.client.post(url, **kwargs)
        # self.logger.info(f'{response.request.method} {url} => {response.status_code}')
        self._assert_status_code(response, code)
        return response

    async def register(self, username: str, password: str, first: str, last: str, income: int,
                       accounttype: Literal['personal', 'business']) -> None:
        await self.get('/session/register')
        response = await self.post('/session/register', data={
            'username': username, 'password': password,
            'firstname': first, 'lastname': last,
            'income': income, 'accounttype': accounttype
        }, follow_redirects=True)
        if response.url.path != '/session/profile':
            self.logger.warning(f'Unexpected redirect to {response.url}')
            raise MumbleException('Registration failed')
        self.logger.info(f'Registered {username!r} (password {password!r})')

    async def api_create(self, database: str, data: Any) -> str:
        response = await self.post(f'/api/{database}/create', json=data)
        self.logger.info(f'KV stored {data} => {response.json()}')
        return response.json()['key']

    async def api_grant(self, database: str, key: str, token: str) -> bool:
        response = await self.post(f'/api/{database}/grant/{key}/{token}')
        self.logger.info(f'KV granted {token} on {key} => {response.json()}')
        return response.json()['ok']

    async def api_get(self, database: str, what: str = 'keys', key: str = 'all', token: str | None = None) -> Any:
        response = await self.get(f'/api/{database}/{what}/{key}', params={'token': token} if token else {})
        return response.json()


async def main() -> None:
    logging.basicConfig(format=f'[%(asctime)s.%(msecs)03d] %(levelname)-9.9s: %(message)s', level=logging.INFO)
    aclient = AsyncClient(base_url='http://localhost:8000')
    client = JitterishClient(logging.LoggerAdapter(logging.getLogger('main')), aclient)
    await client.register('test', 'test', 'First', 'Last', 10000, 'business')


if __name__ == '__main__':
    asyncio.run(main())
