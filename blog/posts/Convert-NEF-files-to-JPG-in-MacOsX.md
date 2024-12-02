---
title: Convert NEF files to JPG in MacOsX
lang: en
date: 2020-10-18 17:20:38
tags:
  - MacOsX
  - Terminal
---
I were a big fan of ametor photografhy and taken too many photos in raw format. A lot. It ware filled up my storage very fast. But actually more than half of the photos are not worth to keep it. I forgot to keep cleaning those files so I did not have enought space to install new software or save some important file.
I started to check one by one each file to which one need to be deleted. But there were to many raw files and need to converted into jpg files.
It is not best choose to use Photoshop to do one by one each photo. Best chose are using batch operation to convert all files. There are too many options to use in terminal.
I preferred to use sips tools for this operation. Actually, sips is very powerful for image processing operation. Also, needs to remove raw files after convert operation.
I used find command to do this operation.

{%- call codeblock('shell') -%}
find . -name "*.NEF" -exec sh -c 'sips -s format jpeg $1 --out "${1%.*}.jpg"' - {} \; -exec rm {} \;
{%- endcall -%}

Maybe it is not a best solution but It worked for me.
