build: 
	cargo build

run: 
	cargo run

test:
	cargo test

push: build
	git add .
	git commit --allow-empty -m "pass stage" 
	git push origin master