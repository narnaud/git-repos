# Changelog

## [0.6.0](https://github.com/narnaud/git-repos/compare/v0.5.0...v0.6.0) (2026-01-30)


### Features ✨

* add Confirmation when dropping a repository ([82324c0](https://github.com/narnaud/git-repos/commit/82324c0d1751191bc6e196eeac39962a4499317c))

## [0.5.0](https://github.com/narnaud/git-repos/compare/v0.4.0...v0.5.0) (2025-12-20)


### Features ✨

* replace 'Needs Attention' filter with 'No Upstream' filter ([55ee4ec](https://github.com/narnaud/git-repos/commit/55ee4ecb75dab863dcad51a9115d983d46b4311d))


### Bug Fixes 🐞

* exclude missing repos from filtered views ([d46ea4c](https://github.com/narnaud/git-repos/commit/d46ea4c9cf5b0ef8c750c7f94f0375bc76a36f03))

## [0.4.0](https://github.com/narnaud/git-repos/compare/v0.3.0...v0.4.0) (2025-12-13)


### Features ✨

* add 'u' shortcut to update selected repo with animation ([054975f](https://github.com/narnaud/git-repos/commit/054975f07e8f428cca4d9a8f10ad565200abcc9a))
* exclude repositories starting with _ from cache ([5d2cd0a](https://github.com/narnaud/git-repos/commit/5d2cd0ac340a5c2f2eeb80105f11b036fdaf489a))


### Bug Fixes 🐞

* manual update now performs fetch with fast-forward ([ec6469b](https://github.com/narnaud/git-repos/commit/ec6469b422614e0e64c340d3a200fb8a4913656e))


### Documentation

* add 'u' shortcut to update selected repo in README ([d8bc377](https://github.com/narnaud/git-repos/commit/d8bc377b6a593b53f83b54d3e3612e1afe91023a))

## [0.3.0](https://github.com/narnaud/git-repos/compare/v0.2.0...v0.3.0) (2025-12-08)


### Features ✨

* add --cwd-file for shell integration ([eb87ede](https://github.com/narnaud/git-repos/commit/eb87ede2eb6e9d3219ebe4d66d7e7fbf71b6daaa))


### Bug Fixes 🐞

* clippy compliance and refactor Enter key logic ([8ff5161](https://github.com/narnaud/git-repos/commit/8ff51617d159e410253afb331c438408c21f1b30))
* strip UNC prefix when saving selected repo path ([2c805b9](https://github.com/narnaud/git-repos/commit/2c805b9ad94ea0f63d96482e4db2d10251ef3545))


### Changes

* centralize UNC path stripping in util.rs ([dab8080](https://github.com/narnaud/git-repos/commit/dab8080330a294a7faf31ec0d99782732e1a09b9))

## [0.2.0](https://github.com/narnaud/git-repos/compare/v0.1.0...v0.2.0) (2025-12-06)


### Features ✨

* add 'd' key to status bar help text ([4a9e0ac](https://github.com/narnaud/git-repos/commit/4a9e0ac339527b700353fc5e4c91c8917ba3aa70))
* add animated deletion progress indicator ([ad7c330](https://github.com/narnaud/git-repos/commit/ad7c330cbfabaffa26084e765e1e205d94242ae0))
* add clone command for missing repositories ([cc6fce7](https://github.com/narnaud/git-repos/commit/cc6fce7bef6d6b15d97eb0ce3874bc3f81c44a80))
* add cloning progress indicator with animation ([d549611](https://github.com/narnaud/git-repos/commit/d54961139012e136f7e31f05a2ec0ef9813f719d))
* add drop repository command with 'd' key ([ffe606e](https://github.com/narnaud/git-repos/commit/ffe606ec7790086b8ba61214f8f36101ae55bcf7))
* add set command to configure default root directory ([072cc53](https://github.com/narnaud/git-repos/commit/072cc538d0d09fa390e53b8cfae0fc52820c3352))
* add update_by_default setting to config ([4bbc25c](https://github.com/narnaud/git-repos/commit/4bbc25c8a559986a3c6eeaa17c38153b5ece9584))
* load and merge repository cache with discovered repos ([e916c92](https://github.com/narnaud/git-repos/commit/e916c9288812070d0166c4f263a1c5fb4fa2414c))
* mark deleted repositories as missing and sort by status ([6fbd0e1](https://github.com/narnaud/git-repos/commit/6fbd0e1e1215b2a980e2c86fc37e562c5cb98dfc))
* save repository cache to YAML when scanning root directory ([134b5dc](https://github.com/narnaud/git-repos/commit/134b5dc8009c27fc96a859744c5d374ad14e0975))
* update submodules after fast-forward merge ([afcc5aa](https://github.com/narnaud/git-repos/commit/afcc5aa46acb30f0dde2577ee93a60a0ec50407a))


### Bug Fixes 🐞

* correct repository index for git status updates after clone ([130691a](https://github.com/narnaud/git-repos/commit/130691a3da8ad389a02b25530cece4d7e7e6fb51))
* improve missing repo visibility when selected ([5bdc4c5](https://github.com/narnaud/git-repos/commit/5bdc4c53ec0192d96823756f0c30539d727a7aeb))
* preserve deleted repositories in cache across restarts ([5fa4c06](https://github.com/narnaud/git-repos/commit/5fa4c067fa6214593c624e916ce2e38e065cbc9c))
* preserve remote URL when deleting repositories ([bcda150](https://github.com/narnaud/git-repos/commit/bcda15075e4405c2122d49419ba507eae20b9d94))
* resort repositories after cloning ([f1702b4](https://github.com/narnaud/git-repos/commit/f1702b4dbc566de023cbf3b5fbbaf45e30df305f))


### Documentation

* update README with latest features and UI changes ([8f50196](https://github.com/narnaud/git-repos/commit/8f50196a68e198e70674d8cb3024dcee4791157c))
* update README with new features ([c957d07](https://github.com/narnaud/git-repos/commit/c957d0706b92d94dd96250c1219f5c768798de95))


### Changes

* clean up GitRepo implementation ([8d716bb](https://github.com/narnaud/git-repos/commit/8d716bbe2a38c0742ae54363e84629de76f5304a))
* extract cache logic into dedicated module ([7129ec2](https://github.com/narnaud/git-repos/commit/7129ec272fac9d24d576be6abd546c60bbd58588))
* extract helper functions from main for better code organization ([1db28aa](https://github.com/narnaud/git-repos/commit/1db28aa7d9ab100040ed5718bac5b4778e50b2d3))
* improve app.rs readability ([fbc26f8](https://github.com/narnaud/git-repos/commit/fbc26f80f2435f69a6ec182b726e5cafc34798d3))
* simplify cache architecture ([562b81c](https://github.com/narnaud/git-repos/commit/562b81ce68c3cebd6c9d2b99f25595c951fca264))
* simplify find_git_repos to use functional iterator chain ([97d4b39](https://github.com/narnaud/git-repos/commit/97d4b391d18db9569554ae37542a616486cad1e9))
* simplify main.rs repo loading logic ([162c911](https://github.com/narnaud/git-repos/commit/162c911a7f4547a88de9ab2e215cd0b567fe9d87))

## 0.1.0 (2025-12-04)


### Features ✨

* add --update flag for fast-forward merge ([0f03791](https://github.com/narnaud/git-repos/commit/0f0379196c9f6c6f109ed40c4a15937c1ef34768))
* add animated status bar for fetch progress ([55141fb](https://github.com/narnaud/git-repos/commit/55141fbc41bbe96b6c7c4bb581227181dc8d2fd2))
* add auto-fetch for repositories with remotes ([4fc0e95](https://github.com/narnaud/git-repos/commit/4fc0e953ad73e4a93d2d1ddf3bcc5fcc9c0ceef5))
* Add branch column to repository table ([6e5e45c](https://github.com/narnaud/git-repos/commit/6e5e45c07e7e73f4ec0727b9f7adb090c9d058d9))
* Add Enter key to navigate to selected repository ([df94ee2](https://github.com/narnaud/git-repos/commit/df94ee28b257b2aa71c66cb039a9b6f44db925a7))
* add filter mode to cycle through repository views ([3ef3a25](https://github.com/narnaud/git-repos/commit/3ef3a25190e1af438a27802e91a1cb0f124369e8))
* Add recursive git repository scanner ([3d50cb9](https://github.com/narnaud/git-repos/commit/3d50cb9ed5bf3f6c6db978cb9dc0b7cec99d2c61))
* Add remote status column with cached data ([d8d697a](https://github.com/narnaud/git-repos/commit/d8d697a47ae478a689f9699b81775bb87c166a35))
* add search functionality with '/' key ([2a45e25](https://github.com/narnaud/git-repos/commit/2a45e25fb221a55fd8e1a980e570cec5a68bdb01))
* Add TUI with ratatui for displaying git repositories ([d1163dd](https://github.com/narnaud/git-repos/commit/d1163dd7481af742047d1688058e406aea32cb14))
* Add working tree status column ([912b1f6](https://github.com/narnaud/git-repos/commit/912b1f68767a7f162b1890188808a7f68cc6bd75))
* auto-merge with fast-forward after fetch ([65ea565](https://github.com/narnaud/git-repos/commit/65ea56592ab8c09c2bf1a91a1cb1c59d08282c8f))
* Load git data asynchronously with placeholders ([2c7e76f](https://github.com/narnaud/git-repos/commit/2c7e76f4119b0e810f190a1931e54a3493c87da2))
* Simplify output to show parent/repo format ([2a038ca](https://github.com/narnaud/git-repos/commit/2a038ca00ccbdd70ca49ffac23cc53ad43c2485a))


### Bug Fixes 🐞

* Collapse nested if statement in event handler ([dc28f14](https://github.com/narnaud/git-repos/commit/dc28f1467c7db3094a0bfbc4195b8c21a400f0b2))


### Documentation

* Add comprehensive README with vibe-coding experiment introduction ([951ebe5](https://github.com/narnaud/git-repos/commit/951ebe5a10abfd8457dea87754541ecd4f7d0c40))
* Add shell integration guide and update features ([8c3fa80](https://github.com/narnaud/git-repos/commit/8c3fa80d92b0cf7826292c3c105cf5837e69c364))
* update README with auto-fetch documentation ([19102c7](https://github.com/narnaud/git-repos/commit/19102c70b80477722de5d4d194bdaf6e42aae4c1))


### Changes

* Create GitRepo struct and move logic to git_repo module ([c196a00](https://github.com/narnaud/git-repos/commit/c196a009311c4a06ad7c6473fae6f5af5f28c8a5))
* Move event handling to dedicated module with async streams ([82a0670](https://github.com/narnaud/git-repos/commit/82a0670496a6becc8b70bab5c0b3e5d5186f580d))
* move filter modes to bottom right and rename to Mode ([4a41ca5](https://github.com/narnaud/git-repos/commit/4a41ca51c7a52ebbb37d68d02fe2b46eda728822))
* Separate App state from UI rendering logic ([6b6f97d](https://github.com/narnaud/git-repos/commit/6b6f97d90fb62cff7230367f41388c28d0f67ead))
* simplify status bar in search mode ([4e4be5a](https://github.com/narnaud/git-repos/commit/4e4be5a7253c6d3716ac32fdadab506cea685d9d))
