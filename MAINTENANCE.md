## Creating a release

* Create PR:
  * ```release-plz release-pr --git-token $GITHUB_QCP_TOKEN```
  * _if this token has expired, you'll need to generate a fresh one; walk back through the release-plz setup steps_
* Merge the PR (rebase strategy preferred)
* Delete the PR branch
* `git fetch && git merge --ff-only`
* Finalise the release:
  * ```release-plz release --git-token $GITHUB_QCP_TOKEN```
* Merge `dev` into `main`, or whatever suits the current branching strategy
* Check the docs built, follow up on the release workflow, etc.
