---
name: Bug report
about: Create a report to help us improve
title: ''
labels: bug
assignees: jonasthewolf

---

**Describe the bug**

A clear and concise description of what the bug is, i.e. the actual behavior.

**To Reproduce**

This is a minimal YAML that reproduces the example:

```yaml
G1:
  text: buggy
```

You can sanitize your files from your intellectual property using, e.g. https://mikefarah.gitbook.io/yq/

```
 yq "(.[] | select(. | has(\"text\"))) .text |=sub(\"[a-zA-Z0-9]\",\"x\"), ... comments=\"\""  inputfile.yaml > outputfile.yaml
```

**Expected behavior**

A clear and concise description of what you expected to happen.

**Screenshots**

If applicable, add screenshots to help explain your problem.

**Desktop (please complete the following information):**

 - OS: [e.g. macOS, Windows, Linux]
 - OS Version: [e.g. 11]
 - SVG Viewer: [e.g. browser Chrome, Safari that is used for viewing the SVG]
 - gsn2x Version: [use `gsn2x --version`]

**Additional context**

Add any other context about the problem here.
