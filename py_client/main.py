import json
import asyncio

from anchorpy import Provider, Wallet, Program, Context, Idl
from solana.system_program import create_account, CreateAccountParams
from solana.rpc.async_api import AsyncClient
from solana.publickey import PublicKey
from solana.keypair import Keypair
from solana.transaction import Transaction

from spl.token.client import Token
from spl.token.instructions import get_associated_token_address, transfer, TransferParams


PROGRAM_ID = "AM8FqXXJeknhcoHk2im2vqzbkveLVq1GSr2kvXsoKRkR"
SYSTEM_PROGRAM_ID = '11111111111111111111111111111111'
ASSOCIATED_TOKEN_ACCOUNT_PROGRAM = "ATokenGPvbdGVxr1b2hvZbsiqW5xWH25efTNsLJA8knL"
TOKEN_PROGRAM_ID = "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA"
RENT_PROGRAM_ID = "SysvarRent111111111111111111111111111111111"

ITEM = "3peupg9t9qxfhX9eqP1bkkdD7zh3y7bDVxG1L5J3gBQM"
TOKEN = "3cXUZuyF2TdUNESFtW7rJDwtYnP5j272mwz9CqpyanqL"

def get_keypair(keypath):
    with open(keypath) as f:
        kpb = json.loads(f.read())
    return Keypair.from_secret_key(secret_key=bytes(kpb))


def get_market_pda(creator_key, item, token):
    result = PublicKey.find_program_address(
        [b'rogue_swapper', bytes(creator_key.public_key), bytes(PublicKey(item)), bytes(PublicKey(token))],
        PublicKey(PROGRAM_ID)
    )
    # 0 is the address
    # 1 is the nonce
    return (result[0], result[1])

async def init_market(provider, idl, payer, market, nonce, item, token):
    ta = get_associated_token_address(payer.public_key, PublicKey(TOKEN) )
    ia = get_associated_token_address(market, PublicKey(ITEM) )
    cia = get_associated_token_address(payer.public_key, PublicKey(ITEM))
    rogue_swapper = Program(idl=idl, program_id=PROGRAM_ID, provider=provider)
    result = await rogue_swapper.rpc["init_market"](
                                                        nonce,
                                                        100,
                                                        2,
                                                        ctx=Context(
                                                           accounts={
                                                               "creator": payer.public_key,
                                                               "item": PublicKey(item),
                                                               "token": PublicKey(token),
                                                               "market": market,
                                                               "item_associated_account": ia,
                                                               "creator_token_associated_account": ta,
                                                               "creator_item_associated_account": cia,
                                                               "token_program": PublicKey(TOKEN_PROGRAM_ID) ,
                                                               "associated_token_program": PublicKey(ASSOCIATED_TOKEN_ACCOUNT_PROGRAM),
                                                               "rent": PublicKey(RENT_PROGRAM_ID),
                                                               "system_program": PublicKey(SYSTEM_PROGRAM_ID)
                                                           },
                                                           signers=[payer]
                                                       )
            )
    print(result)


async def activate_market(provider, idl, payer, market, nonce, item, token):
    rogue_swapper = Program(idl=idl, program_id=PROGRAM_ID, provider=provider)
    result = await rogue_swapper.rpc["edit_market"](
                                                        nonce,
                                                        True,
                                                        None,
                                                        ctx=Context(
                                                           accounts={
                                                               "creator": payer.public_key,
                                                               "item": PublicKey(item),
                                                               "token": PublicKey(token),
                                                               "market": market,
                                                           },
                                                           signers=[payer]
                                                       )
            )
    print(result)

async def buy_item(provider, idl, creator, payer, market, nonce, item, token):
    ia = get_associated_token_address(market, PublicKey(ITEM) )
    bia = get_associated_token_address(payer.public_key, PublicKey(ITEM))
    cta = get_associated_token_address(creator, PublicKey(TOKEN))
    bta = get_associated_token_address(payer.public_key, PublicKey(TOKEN) )
    rogue_swapper = Program(idl=idl, program_id=PROGRAM_ID, provider=provider)
    result = await rogue_swapper.rpc["buy_item"](
                                                        nonce,
                                                        4,
                                                        ctx=Context(
                                                           accounts={
                                                               "buyer": payer.public_key,
                                                               "creator": creator, 
                                                               "item": PublicKey(item),
                                                               "token": PublicKey(token),
                                                               "market": market,
                                                               "item_associated_account": ia,
                                                               "creator_token_associated_account": cta,
                                                               "buyer_token_associated_account":bta,
                                                               "buyer_item_associated_account":bia,
                                                               "token_program": PublicKey(TOKEN_PROGRAM_ID) ,
                                                               "associated_token_program": PublicKey(ASSOCIATED_TOKEN_ACCOUNT_PROGRAM),
                                                               "rent": PublicKey(RENT_PROGRAM_ID),
                                                               "system_program": PublicKey(SYSTEM_PROGRAM_ID)
                                                           },
                                                           signers=[payer]
                                                       )
            )
    print(result)


async def main():
    main_keypair = get_keypair("/Users/rohan/.solw/dub.json")
    async_client = AsyncClient(endpoint="http://127.0.0.1:8899")
    provider = Provider(async_client, Wallet(payer=main_keypair))

    with open("/Users/rohan/anker/target/idl/anker.json") as f:
        raw_idl = f.read()
    idl = Idl.from_json(json.loads(raw_idl))

    (market_pda, nonce) = get_market_pda(main_keypair,ITEM ,TOKEN)
    print(market_pda)
    print(nonce)
    ta = get_associated_token_address(market_pda, PublicKey(TOKEN) )
    ia = get_associated_token_address(market_pda, PublicKey(ITEM) )

    print("item: %s"%(ia))
    print("token: %s"%(ta))
#    await init_market(provider, idl, main_keypair, market_pda, nonce, ITEM, TOKEN)
#    await activate_market(provider, idl, main_keypair, market_pda, nonce, ITEM, TOKEN)

    buyer = get_keypair("/Users/rohan/.solw/king.json")
    await buy_item(provider, idl, main_keypair.public_key, buyer, market_pda, nonce, ITEM, TOKEN)

asyncio.run(main())
