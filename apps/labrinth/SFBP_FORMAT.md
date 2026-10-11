# ShroudForge Blueprint Format (SFBP) V7

ShroudEdit stores and distributes the text blueprint exported by the ShroudForge World Editor. The file extension is `.sfbp`; it is the file format from the beginning of this integration. “Schematic” remains the ShroudEdit project category name, while the uploaded artifact is always `.sfbp`.

## File layout

Each file is UTF-8 text with one value per line:

```text
SHROUDFORGE_WORLD_BLUEPRINT_V7
x|y|z
extent-x,extent-y,extent-z
voxel|props
voxel-width,voxel-height,voxel-depth | -
voxel-offset-x,voxel-offset-y,voxel-offset-z | -
cell-size-x,cell-size-y,cell-size-z | -
voxel-cells | -
voxel-coverage | -
prop-count
item-id,x,y,z,qx,qy,qz,qw,sx,sy,sz,template-uuid-high,template-uuid-low
...
```

Voxel rows contain comma-separated unsigned 16-bit values and a parallel coverage row with values from `0` to `2`. Props-only files use `-` for all five voxel-channel lines. Each prop row has eleven finite numeric values followed by two hexadecimal template UUID parts.

## Validation and metadata

- The extension is `.sfbp` and the first line is exactly `SHROUDFORGE_WORLD_BLUEPRINT_V7`.
- The format version is fixed at `7`.
- The rotation axis is `x`, `y`, or `z`; extent values are finite and non-negative. Props-only extents must be positive.
- Voxel dimensions are positive integers whose product is at most `65,536`. Cell size values are positive, and each voxel and coverage row must contain exactly one entry per cell.
- Prop count is from `0` through `1,000,000`; every row must contain thirteen fields with a positive integer item ID, finite transform values, a non-zero quaternion, and valid non-zero template UUID data.
- The API limits an SFBP file to `95 MiB` and checks its embedded dimensions against the version metadata.
- Version metadata also records the World Editor version, exactly one Enshrouded game version, and optional import instructions. The file itself remains unchanged when stored.
- Compatible loaders are Shroudtopia, ShroudForge, and EML.

Labrinth calculates SHA-1 and SHA-512 checksums for the complete `.sfbp` file. `.mrpack` remains a separate format for modpacks.

## Desktop client authorization

A desktop uploader should use the ShroudEdit OAuth authorization-code flow with S256 PKCE. Open the ShroudEdit authorization page with the registered `client_id`, exact `redirect_uri`, requested scopes, random `state`, `code_challenge`, and `code_challenge_method=S256`. Keep the matching `code_verifier` on the device and exchange the returned code at the OAuth token endpoint. The token request includes `grant_type=authorization_code`, `client_id`, the same `redirect_uri`, the authorization `code`, and `code_verifier`; a client secret is not sent for a PKCE flow.

Use a bearer access token for version API calls. `VERSION_CREATE` permits creating a new blueprint version, and `VERSION_WRITE` permits changing an existing version's files. The API still checks the connected account's project permissions and validates the `.sfbp` file and its metadata on upload.
