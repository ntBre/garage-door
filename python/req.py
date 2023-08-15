import requests

data = b'\x82\xa4meta\x82\xa7include\xc0\xa7exclude\xc0\xa4data\x82\xaacollection\xb3torsiondrivedataset\xa4name\xd96OpenFF multiplicity correction torsion drive data v1.1'
r = requests.get("https://api.qcarchive.molssi.org:443/collection", data=data, headers={'Content-Type': 'application/msgpack-ext'})
print(r.request.url)
print(r.request.headers)
print(r.request.body)
print(r)
