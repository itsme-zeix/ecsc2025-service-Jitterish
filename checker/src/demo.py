import asyncio
import logging
import os
from hashlib import sha256

from enochecker3 import PutflagCheckerTaskMessage, GetflagCheckerTaskMessage, ExploitCheckerTaskMessage, \
    PutnoiseCheckerTaskMessage, GetnoiseCheckerTaskMessage, HavocCheckerTaskMessage, BaseCheckerTaskMessage

from checker import checker

os.environ['MONGO_USER'] = os.environ.get('MONGO_USER', 'jitterish_checker')
os.environ['MONGO_PASSWORD'] = os.environ.get('MONGO_PASSWORD', 'jitterish_checker')


class DemoRunner:
    def __init__(self):
        self.responses = []

    async def run(self, task):
        response = await getattr(checker, f'_call_{task.method.value}')(task)
        print("====================")
        print(response)
        print("====================\n\n")
        self.responses.append((task, response))
        return response

    def report(self) -> None:
        print("\n\n====================")
        for t, r in self.responses:
            print(f'{t.method.value:8s} {t.variant_id}  {r}')

    async def demo(self) -> None:
        """what a mess"""

        task: BaseCheckerTaskMessage

        # setup code
        logging.basicConfig(format=f'[%(asctime)s.%(msecs)03d] %(levelname)-9.9s: %(message)s', level=logging.INFO)
        await checker._init()
        params = {
            'task_id': 1,
            'address': '127.0.0.1',
            'team_id': 1,
            'team_name': 'test',
            'current_round_id': 1,
            'variant_id': 2,
            'timeout': 30000,
            'round_length': 120,
            'task_chain_id': '',
        }
        flag = 'FLAG{TESTFLAG}'

        # whatever you want to test
        #r'''
        task = PutflagCheckerTaskMessage(flag=flag, **params)
        put_response = await self.run(task)
        task = GetflagCheckerTaskMessage(flag=flag, related_round_id=1, **params)
        await self.run(task)
        task = ExploitCheckerTaskMessage(flag_regex=r'FLAG\{.+?\}',
                                         flag_hash=sha256(flag.encode()).hexdigest(),
                                         attack_info=put_response.attack_info,
                                         **params)
        await self.run(task)
        #'''

        await self.run(PutnoiseCheckerTaskMessage(**params))
        await self.run(GetnoiseCheckerTaskMessage(related_round_id=1, **params))
        await self.run(HavocCheckerTaskMessage(**params))

        self.report()


if __name__ == '__main__':
    asyncio.run(DemoRunner().demo())
