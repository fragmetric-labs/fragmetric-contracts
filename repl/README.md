# REPL

## Setup
```
# ensure dependency installation in project root.
../ $ yarn

# install Jupyter Notebook in machine
$ pip install notebook

# install Typescript Kernel in machine
$ sudo -H npm install -g itypescript

# register Typescript Kernel in machine
$ sudo -H its --install=global && jupyter kernelspec list
Available kernels:
  python3       /usr/local/share/jupyter/kernels/python3
  typescript    /usr/local/share/jupyter/kernels/typescript
```

## Run
```
# Prepare target program IDL (with target feature flag; like mainnet)
../ $ anchor build -p restaking

# Run notebook in this direcory.
./ $ jupyter notebook
```

# Contribution Guide
- itypescript ref: https://github.com/winnekes/itypescript/blob/master/doc/usage.md
