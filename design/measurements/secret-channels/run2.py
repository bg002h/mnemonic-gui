import json
from concurrent.futures import ThreadPoolExecutor
from cases2 import C2, measure
with ThreadPoolExecutor(8) as ex: res=list(ex.map(measure,C2))
json.dump(res,open("channels2.json","w"),indent=1)
for r in res:
    print(f"## {r['label']}  base_exit={r['base_exit']} depends={r['depends']} {r['base_err']}")
    for k,v in r["channels"].items(): print(f"     {k:40s} {v}")
