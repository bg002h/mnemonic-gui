import json
from concurrent.futures import ThreadPoolExecutor
from cases3 import C3, measure
with ThreadPoolExecutor(12) as ex: res=list(ex.map(measure,C3))
json.dump(res,open("channels3.json","w"),indent=1)
for r in res:
    print(f"## {r['label']}  base_exit={r['base_exit']} depends={r['depends']} {r['base_err']}")
    for k,v in r["channels"].items(): print(f"     {k:40s} {v}")
