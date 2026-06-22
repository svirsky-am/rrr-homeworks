import json
import base58

with open("/root/.config/solana/id.json") as f:
    secret = json.load(f)

private_key_base58 = base58.b58encode(bytes(secret)).decode()
print("Private key (Base58):")
print(private_key_base58)