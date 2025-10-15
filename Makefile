run:
	cargo run

new-user:
	@curl -X POST "localhost:8080/subscriptions" \
	-H "Content-Type: application/x-www-form-urlencoded" \
	-d "name=henrique%20Veronez&email=henrique@gmail.com"


