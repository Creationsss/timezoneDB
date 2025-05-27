import { echo } from "@atums/echo";

const environment: Environment = {
	port: Number.parseInt(process.env.PORT || "8080", 10),
	host: process.env.HOST || "0.0.0.0",
	development:
		process.env.NODE_ENV === "development" || process.argv.includes("--dev"),
};

const discordConfig = {
	clientId: process.env.CLIENT_ID || "",
	clientSecret: process.env.CLIENT_SECRET || "",
	redirectUri: process.env.REDIRECT_URI || "",
};

function verifyRequiredVariables(): void {
	const requiredVariables = [
		"HOST",
		"PORT",

		"PGHOST",
		"PGPORT",
		"PGUSERNAME",
		"PGPASSWORD",
		"PGDATABASE",

		"REDIS_URL",

		"CLIENT_ID",
		"CLIENT_SECRET",
		"REDIRECT_URI",
	];

	let hasError = false;

	for (const key of requiredVariables) {
		const value = process.env[key];
		if (value === undefined || value.trim() === "") {
			echo.error(`Missing or empty environment variable: ${key}`);
			hasError = true;
		}
	}

	if (hasError) {
		process.exit(1);
	}
}

export { environment, discordConfig, verifyRequiredVariables };
