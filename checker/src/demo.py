import asyncio
import logging
from hashlib import sha256

from enochecker3 import PutflagCheckerTaskMessage, GetflagCheckerTaskMessage, ExploitCheckerTaskMessage

from checker import checker


async def demo() -> None:
    """what a mess"""

    # setup code
    logging.basicConfig(format=f'[%(asctime)s.%(msecs)03d] %(levelname)-9.9s: %(message)s', level=logging.INFO)
    await checker._init()
    params = {
        'task_id': 1,
        'address': '127.0.0.1',
        'team_id': 1,
        'team_name': 'test',
        'current_round_id': 1,
        'variant_id': 0,
        'timeout': 30000,
        'round_length': 120,
        'task_chain_id': '',
    }
    flag = 'FLAG{...}'

    # whatever you want to test
    task = PutflagCheckerTaskMessage(flag=flag, **params)
    response = await checker._call_putflag(task)
    print(response)

    task = GetflagCheckerTaskMessage(flag=flag, related_round_id=1, **params)
    print(await checker._call_getflag(task))

    task = ExploitCheckerTaskMessage(flag_regex=r'FLAG\{.+?\}',
                                     flag_hash=sha256(flag.encode()).hexdigest(),
                                     attack_info=response.attack_info,
                                     **params)
    print(await checker._call_exploit(task))


if __name__ == '__main__':
    asyncio.run(demo())