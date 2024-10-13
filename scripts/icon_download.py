from steam.client import SteamClient

client = SteamClient()
client.anonymous_login()
client.verbose_debug = True

for i in range(60):
    try:
        info = client.get_product_info(apps=[440], timeout=1)
        print(info)
    except:
        print(f'failed {i}')