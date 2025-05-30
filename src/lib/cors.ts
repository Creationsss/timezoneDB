function withCors(response: Response, req: Request): Response {
	const origin = req.headers.get("origin");

	const headers = new Headers(response.headers);
	if (origin) {
		headers.set("Access-Control-Allow-Origin", origin);
		headers.set("Access-Control-Allow-Credentials", "true");
		headers.set("Vary", "Origin");
	}

	return new Response(response.body, {
		status: response.status,
		statusText: response.statusText,
		headers,
	});
}

export { withCors };
