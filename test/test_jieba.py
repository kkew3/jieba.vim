import os
from typing import Literal
import subprocess

from jinja2 import Environment, FileSystemLoader

BASEDIR = 'cases'


def clean_vader():
    for name in os.listdir(BASEDIR):
        if name.endswith('.vader'):
            os.remove(os.path.join(BASEDIR, name))


def compile_j2(vim_bin: Literal['vim', 'nvim'], verify_only: bool):
    env = Environment(loader=FileSystemLoader(BASEDIR))
    vim = (vim_bin == 'vim')
    nvim = (vim_bin == 'nvim')
    for name in os.listdir(BASEDIR):
        if name.endswith('.vader.j2'):
            template = env.get_template(name)
            file = 'verify_{}'.format(name[:-3])
            with open(
                    os.path.join(BASEDIR, file), 'w',
                    encoding='utf-8') as outfile:
                outfile.write(template.render(vim=vim, nvim=nvim, verify=True))
            if not verify_only:
                file = 'test_{}'.format(name[:-3])
                with open(
                        os.path.join(BASEDIR, file), 'w',
                        encoding='utf-8') as outfile:
                    outfile.write(
                        template.render(vim=vim, nvim=nvim, verify=False))


def eval_with_vim(vim_bin: str):
    proc = subprocess.run([
        vim_bin, '-u', 'vimrc', '-c', 'silent Vader! {}'.format(
            os.path.join(BASEDIR, '*.vader'))
    ],
                          capture_output=True,
                          text=True)
    if proc.returncode != 0:
        assert False, proc.stderr


def test_all():
    vim_bin = os.environ['VIM_BIN_NAME']
    verify_only = bool(int(os.environ.get('VERIFY_ONLY', '0')))
    _, _, vim_bin_name = vim_bin.rpartition('/')
    assert vim_bin_name in ('vim', 'nvim')
    clean_vader()
    compile_j2(vim_bin_name, verify_only)
    eval_with_vim(vim_bin)
