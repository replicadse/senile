#!/usr/bin/env python
import fileinput
import json
from pprint import pprint

content = ''
for line in fileinput.input():
    content += line

data = json.loads(content)
#pprint(data)

data_assignee = {}
data_prio = {}

for d in data:
    data_assignee[d['assignee']] = []
    data_prio[d['priority']] = []
for d in data:
    data_assignee[d['assignee']].append(d)
    data_prio[d['priority']].append(d)

def print_todo(d):
    print('- prio:\t\t{}\n  assignee:\t{}\n  body:\t\t{}\n  context:\t{}\n  file:\t\t{}\n  line:\t\t{}'
            .format(d['priority'], d['assignee'], d['body'], '\t\t'.join(d['context']).strip('\n').strip(), d['file'], d['line']))

for assignee in data_assignee.keys():
    print('### {}'.format(assignee))
    print()
    v = data_assignee[assignee]
    v = sorted(v, key=lambda x: x['priority'])
    for d in v:
        print_todo(d)

