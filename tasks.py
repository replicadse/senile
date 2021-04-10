from invoke import task, Collection 

@task
def task_build(c, release=False):
    flags = []
    if release:
        flags.append('--release')
    c.run('cargo build ' + ' '.join(flags))

@task
def task_format(c):
    c.run('cargo fmt --all')

@task
def task_scan(c):
    c.run('cargo fmt --all -- --check')

@task
def task_other_installhooks(c):
    c.run('rm -rf .git/hooks')
    c.run('ln -s ./scripts/git-hooks .git/hooks')
    c.run('chmod -R +x ./scripts/*')

@task
def task_ci_updateversion(c, version):
    c.run(f'''sed 's/version = "0.0.0"/version = "'{version}'"/g' Cargo.toml > Cargo.toml.tmp''')
    c.run('mv Cargo.toml.tmp Cargo.toml')
    c.run(f'''sed 's/pkgver=0.0.0/pkgver='{version}'/g' pkg/aur/PKGBUILD > pkg/aur/PKGBUILD.tmp''')
    c.run('mv pkg/aur/PKGBUILD.tmp pkg/aur/PKGBUILD')
 

ns = Collection()
# TODO!(min, aw, 3): group these into a group?
ns.add_task(task_build, 'build')
ns.add_task(task_format, 'fmt')
ns.add_task(task_scan, 'scan')

ns_other = Collection('other')
ns_other.add_task(task_other_installhooks, 'install-hooks')
ns.add_collection(ns_other, 'other')

ns_ci = Collection('ci')
ns_ci.add_task(task_ci_updateversion, 'update-version')
ns.add_collection(ns_ci, 'ci')

