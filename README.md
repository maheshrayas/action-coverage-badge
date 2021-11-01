
<h1 align="center">
  <p align="center">action-coverage-badger</p>
</h1>

<div align="center">
  <a href="https://github.com/maheshrayas/action-coverage-badge/actions" alt="Build"><img src="https://github.com/maheshrayas/action-coverage-badge/workflows/build/badge.svg" /></a>
  <a href="https://github.com/maheshrayas/action-coverage-badge/actions/workflows/lint.yaml" alt="Lint"><img src="https://github.com/maheshrayas/action-coverage-badge/actions/workflows/lint.yaml/badge.svg" /></a>
  <a href="https://github.com/maheshrayas/action-coverage-badge/commits/main" alt="last commit"><img src="https://img.shields.io/github/last-commit/maheshrayas/action-coverage-badge?color=purple" /></a>
  <a alt="Action pulled"><img src="https://img.shields.io/docker/pulls/maheshrayas/coverage-badger.svg" /></a>
</div>

## Motivation

Img shields can display coverage badge when you pass the percentage in the url (https://img.shields.io/badge/coverage-24%25-red). This URL can be pasted in README.md in html tag `<img>` which would display a Github Badge. Since the url is static with the coverage percentage hardcoded we needed a mechanism to dynamically update the test coverage % whenever a code is added or removed.

This GHA will be triggered whenever a change is pushed to default branch. It reads the cover.json, updates the coverage % in the README.md, create a PR to default branch and automerges.


## Configuration in GHA workflow

Refer to the example on how to setup the github action
https://github.com/maheshrayas/go-badger-test

```bash
      - name: coverage badge
        uses: maheshrayas/action-coverage-badge@v1
        with:
          token: '${{ secrets.GITHUB_TOKEN }}'
          user: <github_username>
          email: <github_email>
```

## Supported Languages

* Go

## TODO

* Support other languages
