build:
	wasm-pack build --target web

serve:
	http-server -p 4000 --cors
