import json, sys
from concurrent.futures import ThreadPoolExecutor
from cases import C, measure
with ThreadPoolExecutor(16) as ex: res=list(ex.map(measure,C))
json.dump(res,open("channels.json","w"),indent=1)
for r in res:
    print(f"## {r['label']}  base_exit={r['base_exit']} depends={r['depends']} {r['base_err']}")
    for k,v in r["channels"].items(): print(f"     {k:40s} {v}")
