#!/usr/bin/env python
import fileinput
import json

content = ''
for line in fileinput.input():
    content += line

data = json.loads(content)

data_assignee = {}
data_prio = {}

for d in data:
    data_assignee[d['assignee']] = []
    data_prio[d['priority']] = []
for d in data:
    data_assignee[d['assignee']].append(d)
    data_prio[d['priority']].append(d)

def print_todo(d):
    print('- prio: `{}` +'.format(d['priority']))
    print('  assignee: `{}` +'.format(d['assignee']))
    print('  body: `{}` +'.format(d['body']))
    print('  context: \n\t```\n\t{}\n\t``` +'.format('\n\t'.join([l.strip().strip('\n') for l in d['context']])))
    print('  file: `{}` +'.format(d['file']))
    print('  line: `{}`'.format(d['line']))

for assignee in data_assignee.keys():
    print('=== {}'.format(assignee))
    print('')
    v = data_assignee[assignee]
    v = sorted(v, key=lambda x: x['priority'])
    for d in v:
        print_todo(d)
