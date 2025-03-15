# ETL pipelines for LLM Analytics

## Quick Start

- TODO

## Check Generated Rules

```bash
cat ../etc/botwaf.yaml | yq e '.botwaf.static-rules[].value' > /tmp/modsecurity.rules
modsec-rules-check /tmp/modsecurity.rules
# : ../etc/modsecurity.rules  --  Loaded 18 rules.
#Test ok.
```
