# 1.0.1 (2019-01-14)

* Disable HTTP keep-alive. This might have been causing some issues when
  running in AWS behind an ELB.
  * Only way to be sure is to try it. It is unlikely something that needs to be
    enabled anyway for a daemon like this.
  * If you are sad about this, it's possible to enable it again and have a
    switch to enable/disable it.
