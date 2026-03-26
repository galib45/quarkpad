## Building the flatpak
```bash
flatpak-builder --force-clean --install-deps-from=flathub --gpg-sign="Asadul Al Galib" --repo=repo build build-aux/flatpak-manifest.yml
flatpak build-bundle --gpg-sign="Asadul Al Galib" repo quarkpad.flatpak org.galib.quarkpad
flatpak build-update-repo --gpg-sign="Asadul Al Galib" --generate-static-deltas --prune repo
```

## Deploying to netlify
```bash
rm -rf site
mkdir site
mv repo site/
cp build-aux/index.html site/
netlify deploy --prod --no-build -d site
```
