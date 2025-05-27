import { discordConfig } from "@config";
import { withCors } from "@lib/cors";
import { randomUUIDv7, redis } from "bun";

export class DiscordAuth {
	#clientId = discordConfig.clientId;
	#clientSecret = discordConfig.clientSecret;
	#redirectUri = discordConfig.redirectUri;

	startOAuthRedirect(req: Request): Response {
		const query = new URLSearchParams({
			client_id: this.#clientId,
			redirect_uri: this.#redirectUri,
			response_type: "code",
			scope: "identify",
		});
		return withCors(
			Response.redirect(`https://discord.com/oauth2/authorize?${query}`, 302),
			req,
		);
	}

	async handleCallback(req: Request): Promise<Response> {
		const url = new URL(req.url);
		const code = url.searchParams.get("code");
		if (!code)
			return withCors(
				Response.json({ error: "Missing code" }, { status: 400 }),
				req,
			);

		const tokenRes = await fetch("https://discord.com/api/oauth2/token", {
			method: "POST",
			headers: { "Content-Type": "application/x-www-form-urlencoded" },
			body: new URLSearchParams({
				client_id: this.#clientId,
				client_secret: this.#clientSecret,
				grant_type: "authorization_code",
				code,
				redirect_uri: this.#redirectUri,
			}),
		});

		const tokenData: { access_token?: string } = await tokenRes.json();
		if (!tokenData.access_token)
			return withCors(
				Response.json({ error: "Unauthorized" }, { status: 401 }),
				req,
			);

		const userRes = await fetch("https://discord.com/api/users/@me", {
			headers: { Authorization: `Bearer ${tokenData.access_token}` },
		});
		const user: DiscordUser = await userRes.json();

		const sessionId = randomUUIDv7();
		await redis.set(sessionId, JSON.stringify(user), "EX", 3600);

		return withCors(
			Response.json(
				{ message: "Authenticated" },
				{
					headers: {
						"Set-Cookie": `session=${sessionId}; HttpOnly; Path=/; Max-Age=3600; SameSite=None; Secure`,
						"Content-Type": "application/json",
					},
				},
			),
			req,
		);
	}

	async getUser(req: Request): Promise<DiscordUser | null> {
		const cookie = req.headers.get("cookie");
		if (!cookie) return null;

		const match = cookie.match(/session=([^;]+)/);
		if (!match) return null;

		const sessionId = match[1];
		const userData = await redis.get(sessionId);
		if (!userData) return null;

		try {
			return JSON.parse(userData);
		} catch {
			return null;
		}
	}
}
