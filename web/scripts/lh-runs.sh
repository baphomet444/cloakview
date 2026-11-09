#!/bin/bash
# Rebuild, arranca next start en :3109 y corre Lighthouse N veces en / y /es.
# uso: lh-runs.sh <prefijo> <n>   (salida en ~/.hermes/cache/scratch/lh/<prefijo><i>-<en|es>.report.{json,html})
# Baseline del estado previo: git stash -q && lh-runs.sh base 3; git stash pop -q
set -u
cd ~/cloakview/web
S=$HOME/.hermes/cache/scratch/lh; mkdir -p "$S"
pkill -f "next start -p 3109"
npm run build > "$S/build.log" 2>&1; echo build_exit=$?
grep -iE "error|warn" "$S/build.log"
(npx next start -p 3109 > "$S/server.log" 2>&1 &); sleep 4
curl -s localhost:3109/ >/dev/null; curl -s localhost:3109/es >/dev/null   # calentar
for i in $(seq 1 "$2"); do for p in "" es; do n=${p:-en}
  lighthouse "http://localhost:3109/$p" --quiet --chrome-flags="--headless=new" \
    --output=json --output=html --output-path="$S/$1$i-$n" >/dev/null 2>&1
  python3 -c "
import json;d=json.load(open('$S/$1$i-$n.report.json'));a=d['audits'];m=a['metrics']['details']['items'][0]
print('run$i $n',{k:round(v['score']*100) for k,v in d['categories'].items() if k!='agentic-browsing'},
 'LCP',a['largest-contentful-paint']['displayValue'],'obsLCP',m['observedLargestContentfulPaint'],
 'TBT',a['total-blocking-time']['displayValue'],
 'console',[x['sourceLocation']['url'][-40:] for x in a['errors-in-console'].get('details',{}).get('items',[])])"
done; done
