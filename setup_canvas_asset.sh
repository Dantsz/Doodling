(cd DoodlingCanvas/; wasm-pack build --no-typescript --no-pack --target=web --dev)
rm -rf DoodlingServer/DoodlingHtmx/pkg
cp -r DoodlingCanvas/pkg DoodlingServer/DoodlingHtmx/resources/pkg
