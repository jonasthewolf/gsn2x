
# Interfacing 

Here is some guidance on interfacing with external sources.

I currently see two use-cases: 

1) Use-case: Check if all configuration management items of a project exist that are mandated by the argumentation (as solutions/evidences).

   The reference list may comprise more items that are required for argumentation, i.e. the list items must be attributed.

   The reference list may be distributed across multiple files.

1) Use-case: Check if all normative/external requirements are fulfilled.

   The reference list is a list of all normative requirements.
   
   This use-case is actually the other way around than the first one.

For the example scripts below, [`yq`](https://github.com/mikefarah/yq) is required.


## Checking evidences

This command lists all Solutions that are in `example.gsn.yaml` but are not in `reference.yaml`:

    yq ea '[select(file_index == 0)|.Sn*.text] - [select(file_index == 1)|.[]]' examples/example.gsn.yaml reference.yaml   

`reference.yaml` could look like this:

    # Configuration Management Items
    - Solution 1
    - Solution 2

## Checking references

Depending on how the external format is defined, you can swap the part before the `-`,
with the part after.

<!-- ## MDG XML -->

