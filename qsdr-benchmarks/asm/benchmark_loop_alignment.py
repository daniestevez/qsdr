#!/usr/bin/env python3

import argparse
import json
import pathlib
import tempfile
import subprocess


# Software Optimization Guide for AMD Family 19h Processors Section 2.8.3.1
nop_table = [
    '90',
    '66 90',
    '0F 1F 00',
    '0F 1F 40 00',
    '0F 1F 44 00 00',
    '66 0F 1F 44 00 00',
    '0F 1F 80 00 00 00 00',
    '0F 1F 84 00 00 00 00 00',
    '66 0F 1F 84 00 00 00 00 00',
    '66 66 0F 1F 84 00 00 00 00 00',
    '66 66 66 0F 1F 84 00 00 00 00 00',
    '66 66 66 66 0F 1F 84 00 00 00 00 00',
    '66 66 66 66 66 0F 1F 84 00 00 00 00 00',
    '66 66 66 66 66 66 0F 1F 84 00 00 00 00 00',
    '66 66 66 66 66 66 66 0F 1F 84 00 00 00 00 00',
]


def load_asm(asm_file):
    with open(asm_file) as f:
        asm = f.readlines()
    loop_idx = asm.index('.loop:\n')
    prologue = ''.join(asm[:loop_idx])
    body = ''.join(asm[loop_idx:])
    return prologue, body


def generate_nops(size):
    nops = []
    while size > 0:
        nop_size = min(size, len(nop_table))
        nop = nop_table[nop_size - 1]
        nop = ', '.join([f'0x{b}' for b in nop.split()])
        nop = f'        db {nop}\n'
        nops.append(nop)
        size -= nop_size
    return ''.join(nops)


def benchmark(nop_padding, args):
    nops = generate_nops(nop_padding)
    prologue, body = load_asm(args.source)
    program = prologue + nops + body
    with tempfile.NamedTemporaryFile(
            mode='w', suffix='.s', delete_on_close=False) as asm_file:
        print(program, file=asm_file)
        asm_file.close()
        subprocess.run(['nasm', '-felf64', asm_file.name])
    objfile = pathlib.Path(asm_file.name).with_suffix('.o')
    exefile = pathlib.Path(asm_file.name).with_suffix('')
    subprocess.run(['ld', '-x', '-o', str(exefile), str(objfile)])
    objfile.unlink()
    perf = subprocess.run(['taskset', args.taskset_mask,
                           'perf', 'stat', '-j', '-e',
                           'cpu-clock,cycles,instructions,'
                           'ic_fetch_stall.ic_stall_any,'
                           'op_cache_hit_miss.all_op_cache_accesses,'
                           'de_dis_uop_queue_empty_di0',
                           str(exefile)],
                          capture_output=True)
    exefile.unlink()
    perf = str(perf.stderr, encoding='ascii')
    perf = '[' + ','.join(perf.strip().split('\n')) + ']'
    perf = json.loads(perf)
    perf = {ev['event']: ev['counter-value'] for ev in perf}
    return perf


def parse_args():
    parser = argparse.ArgumentParser()
    parser.add_argument('--taskset-mask', default='0x1',
                        help='Taskset CPU mask [default=%(default)r]')
    parser.add_argument('--source', required=True,
                        help='Assembly source code for program without loop padding')
    parser.add_argument('--output', required=True,
                        help='Output JSON file')
    return parser.parse_args()


def main():
    args = parse_args()
    perfs = []
    for padding in range(128):
        perf = benchmark(padding, args)
        perf['nop_padding'] = padding
        perfs.append(perf)
    with open(args.output, 'w') as f:
        json.dump(perfs, f)


if __name__ == '__main__':
    main()
