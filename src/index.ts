import { DiscordAuth } from "@/discord";
import { echo } from "@atums/echo";
import { environment, verifyRequiredVariables } from "@config";
import { serve, sql } from "bun";

verifyRequiredVariables();

try {
	await sql`SELECT 1`;
	await sql`
		CREATE TABLE IF NOT EXISTS timezones (
			user_id TEXT PRIMARY KEY,
			username TEXT NOT NULL,
			timezone TEXT NOT NULL,
			created_at TIMESTAMPTZ DEFAULT NOW()
		)
	`;

	echo.info(
		`Connected to PostgreSQL on ${process.env.PGHOST}:${process.env.PGPORT}`,
	);
} catch (error) {
	echo.error({
		message: "Could not establish a connection to PostgreSQL",
		error,
	});
	process.exit(1);
}

try {
	const url = new URL(process.env.REDIS_URL || "redis://localhost:6379");
	echo.info(`Connected to Redis on ${url.hostname}:${url.port || "6379"}`);
} catch (error) {
	echo.error({ message: "Redis connection failed", error });
	process.exit(1);
}

echo.info(`Listening on http://${environment.host}:${environment.port}`);

const auth = new DiscordAuth();

function withCors(res: Response): Response {
	const headers = new Headers(res.headers);
	headers.set("Access-Control-Allow-Origin", "*");
	headers.set(
		"Access-Control-Allow-Methods",
		"GET, POST, PUT, DELETE, OPTIONS",
	);
	headers.set("Access-Control-Allow-Headers", "Content-Type, Authorization");
	headers.set("Access-Control-Allow-Credentials", "true");
	headers.set("Access-Control-Max-Age", "86400");
	return new Response(res.body, {
		status: res.status,
		statusText: res.statusText,
		headers,
	});
}

serve({
	port: environment.port,
	fetch: async (req) => {
		if (req.method === "OPTIONS") {
			return new Response(null, {
				status: 204,
				headers: {
					"Access-Control-Allow-Origin": "*",
					"Access-Control-Allow-Methods": "GET, POST, PUT, DELETE, OPTIONS",
					"Access-Control-Allow-Headers": "Content-Type, Authorization",
					"Access-Control-Max-Age": "86400", // 24 hours
					"Access-Control-Allow-Credentials": "true",
				},
			});
		}

		const url = new URL(req.url);

		if (url.pathname === "/auth/discord") return auth.startOAuthRedirect();
		if (url.pathname === "/auth/discord/callback")
			return auth.handleCallback(req);

		if (url.pathname === "/set") {
			const user = await auth.getUser(req);
			if (!user)
				return withCors(
					Response.json({ error: "Unauthorized" }, { status: 401 }),
				);

			const tz = url.searchParams.get("timezone");
			if (!tz)
				return withCors(
					Response.json(
						{ error: "Timezone parameter is required" },
						{ status: 400 },
					),
				);

			try {
				new Intl.DateTimeFormat("en-US", { timeZone: tz });
			} catch {
				return withCors(
					Response.json({ error: "Invalid timezone" }, { status: 400 }),
				);
			}

			await sql`
				INSERT INTO timezones (user_id, username, timezone)
				VALUES (${user.id}, ${user.username}, ${tz})
				ON CONFLICT (user_id) DO UPDATE
				SET username = EXCLUDED.username, timezone = EXCLUDED.timezone
			`;

			return withCors(Response.json({ success: true }));
		}

		if (url.pathname === "/get") {
			const id = url.searchParams.get("id");
			if (!id)
				return withCors(
					Response.json({ error: "Missing user ID" }, { status: 400 }),
				);

			const rows = await sql`
				SELECT username, timezone FROM timezones WHERE user_id = ${id}
			`;

			if (rows.length === 0) {
				return withCors(
					Response.json({ error: "User not found" }, { status: 404 }),
				);
			}

			return withCors(
				Response.json({
					user: { id, username: rows[0].username },
					timezone: rows[0].timezone,
				}),
			);
		}

		if (url.pathname === "/me") {
			const user = await auth.getUser(req);
			if (!user)
				return withCors(
					Response.json({ error: "Unauthorized" }, { status: 401 }),
				);
			return withCors(Response.json(user));
		}

		return withCors(Response.json({ error: "Not Found" }, { status: 404 }));
	},
});
