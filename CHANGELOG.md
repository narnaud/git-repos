# Changelog

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
