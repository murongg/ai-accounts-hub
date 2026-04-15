export interface GitHubReleaseSummary {
  tag_name?: unknown
  draft?: unknown
  prerelease?: unknown
  assets?: unknown
}

const APP_RELEASE_TAG_PATTERN = /^v\d+\.\d+\.\d+(?:[-+].*)?$/

export function selectLatestAppReleaseTag(releases: GitHubReleaseSummary[]) {
  const release = releases.find((item) => {
    if (item.draft === true || item.prerelease === true) {
      return false
    }

    return typeof item.tag_name === 'string' && APP_RELEASE_TAG_PATTERN.test(item.tag_name)
  })

  return typeof release?.tag_name === 'string' ? release.tag_name : null
}

export function selectLatestAppRelease<T extends GitHubReleaseSummary>(releases: T[]) {
  return (
    releases.find((item) => {
      if (item.draft === true || item.prerelease === true) {
        return false
      }

      return typeof item.tag_name === 'string' && APP_RELEASE_TAG_PATTERN.test(item.tag_name)
    }) ?? null
  )
}
