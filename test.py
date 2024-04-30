# import requests

# cookies = {
#     'l2c_2_pg': 'true',
#     '_csrf-frontend': '3083f91cba9073ff736cbca7072af47a232f9819a01d35179849700389b15c45a%3A2%3A%7Bi%3A0%3Bs%3A14%3A%22_csrf-frontend%22%3Bi%3A1%3Bs%3A32%3A%22EN84DVkUkqMIfVfN-MbMZlc7FA_F86w-%22%3B%7D',
#     'l2c_1': 'true',
# }

# headers = {
#     'User-Agent': 'Mozilla/5.0 (Windows NT 10.0; Win64; x64; rv:125.0) Gecko/20100101 Firefox/125.0',
#     'Accept': 'text/html,application/xhtml+xml,application/xml;q=0.9,image/avif,image/webp,*/*;q=0.8',
#     'Accept-Language': 'en-US,en;q=0.5',
#     # 'Accept-Encoding': 'gzip, deflate, br',
#     'Content-Type': 'multipart/form-data; boundary=---------------------------17997435617953221811232409032',
#     'Origin': 'https://pastebin.com',
#     'Connection': 'keep-alive',
#     'Referer': 'https://pastebin.com/',
#     # 'Cookie': 'l2c_2_pg=true; _csrf-frontend=3083f91cba9073ff736cbca7072af47a232f9819a01d35179849700389b15c45a%3A2%3A%7Bi%3A0%3Bs%3A14%3A%22_csrf-frontend%22%3Bi%3A1%3Bs%3A32%3A%22EN84DVkUkqMIfVfN-MbMZlc7FA_F86w-%22%3B%7D; l2c_1=true',
#     'Upgrade-Insecure-Requests': '1',
#     'Sec-Fetch-Dest': 'document',
#     'Sec-Fetch-Mode': 'navigate',
#     'Sec-Fetch-Site': 'same-origin',
#     'Sec-Fetch-User': '?1',
#     'DNT': '1',
#     'Sec-GPC': '1',
#     # Requests doesn't support trailers
#     # 'TE': 'trailers',
# }

# files = {
#     '_csrf-frontend': (None, 'X-JxMmXWO4Gek8qwPKoTr_L8BoXhxIoMJLMII2klex8arEkGIYBQ1PXih_la_HXh37FkyLuo6Tti8ldlURMMMg=='),
#     'PostForm[text]': (None, 'bruh'),
#     'PostForm[category_id]': (None, '0'),
#     'PostForm[tag]': (None, ''),
#     'PostForm[format]': (None, '1'),
#     'PostForm[expiration]': (None, 'N'),
#     'PostForm[status]': (None, '0'),
#     'PostForm[is_password_enabled]': (None, '0'),
#     'PostForm[is_burn]': (None, '0'),
#     'PostForm[name]': (None, ''),
# }

# response = requests.post('http://localhost:3000/', cookies=cookies, headers=headers, files=files)
# print(response)

import requests
payload = {'username':'niceusername','password':'123456'}

files = {
    '_csrf-frontend': (None, 'X-JxMmXWO4Gek8qwPKoTr_L8BoXhxIoMJLMII2klex8arEkGIYBQ1PXih_la_HXh37FkyLuo6Tti8ldlURMMMg=='),
    'PostForm[text]': (None, 'bruh'),
    'PostForm[category_id]': (None, '0'),
    'PostForm[tag]': (None, ''),
    'PostForm[format]': (None, '1'),
    'PostForm[expiration]': (None, 'N'),
    'PostForm[status]': (None, '0'),
    'PostForm[is_password_enabled]': (None, '0'),
    'PostForm[is_burn]': (None, '0'),
    'PostForm[name]': (None, ''),
}

requests.post('http://localhost:3000/', files=files)
