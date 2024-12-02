---
title: Deleting commit from Mercurial via terminal
tags:
  - Mercurial
  - Source Control
lang: en
date: 2018-10-07 20:38:34
---


Let's assume that you are already have repository.
I am going to create new branch for testing.

{% call codeblock('shell') %}
$ hg branch TestBranch
$ hg commit -m 'You can not delete this commit easly'
{% endcall %}

Now we need to configure (**.hgrc** or **Mercurial.in**) file for enabling strip extension. Add following lines to configuration file.

{% call codeblock('ini') %}
[extensions]
mq =
{% endcall %}

Before deleting last commit we need to get repository information for validating.

{% call codeblock('shell') %}
$ hg sum
parent: 344:9daa6f893e23 tip
made a new branch from revision 500
branch: newbranch
commit: (clean)
update: (current)
phases: 1 draft
{% endcall %}

Also we need changeset id for deleting commit. Our last commit changeset id 9daa6f893e23 and you can easly see on below. Or if you dont want to use hg sum and get directly changeset id, you can execute following command.

{% call codeblock('shell') %}
$ hg id -i
9daa6f893e23

$ hg strip 9daa6f893e23
0 files updated, 0 files merged, 0 files removed, 0 files unresolved
saved backup bundle to C:\Codes\images\.hg\strip-backup/9daa6f893e23-e1200d21-backup.hg
$ hg sum
parent: 342:0ab421d404c2
Use the correct versions of ABC
branch: ABC
commit: (clean)
update: 1 new changesets, 2 branch heads (merge)
{% endcall %}

That's all.