# webfox (wx)

[![Build status](https://github.com/smeets/webfox/workflows/ci/badge.svg)](https://github.com/smeets/webfox/actions)

- [x] Automagic `Content-Type` depending on `--json` (default) or `--form`
- [x] Custom HTTP Header (`X-API-KEY:secret` --> `x-api-key: secret`)
- [x] Query string (`q==search t==secret` --> `?q=search&t=secret`)
- [ ] Body from file (`--data=../file.json`)
- JSON mode
	- [x] Key-value (`q=search` --> `{ "q": "search" }`)
	- [x] Raw (`q=search num:=5` --> `{ "q": "search", "num": 5 }`)
	- [ ] Field from file (`q@=../file.json`)
- Form mode
	- [x] Key-value (`q=search` --> `q=search`)
- Raw mode
	- [ ] `--raw` requires manual content type
- [x] Colorized JSON and HTTP headers output
- [ ] Verbose output option (`-V`)
- [ ] Redirection policy (`--no-follow`, `--follow`)
- [ ] Proxy support (`--proxy {}`)
- [ ] Trust custom root certificates (`--trust-cert {}`)
- [ ] Use client identity (`--identity {}`)