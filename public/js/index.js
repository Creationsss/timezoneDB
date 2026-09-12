const $ = (id) => document.getElementById(id);

const TOM_SELECT = "https://cdn.jsdelivr.net/npm/tom-select@2.6.2/dist";
const browserZone = Intl.DateTimeFormat().resolvedOptions().timeZone;
let ts = null;
let prefer24 = readFormatPreference();
let accountZone = null;
let lookupZone = null;

function readFormatPreference() {
	try {
		const saved = localStorage.getItem("timeFormat24Hour");
		if (saved !== null) return saved === "true";
	} catch {}
	return !new Intl.DateTimeFormat(navigator.language, { hour: "numeric" }).resolvedOptions()
		.hour12;
}

function writeFormatPreference(value) {
	prefer24 = value;
	try {
		localStorage.setItem("timeFormat24Hour", String(value));
	} catch {}
}

function formatTime(zone, date) {
	return new Intl.DateTimeFormat(navigator.language, {
		timeZone: zone,
		hour: prefer24 ? "2-digit" : "numeric",
		minute: "2-digit",
		hour12: !prefer24,
	}).format(date);
}

function formatDate(zone, date) {
	return new Intl.DateTimeFormat(navigator.language, {
		timeZone: zone,
		weekday: "long",
		month: "long",
		day: "numeric",
	}).format(date);
}

function formatOffset(zone, date) {
	const part = new Intl.DateTimeFormat("en-US", {
		timeZone: zone,
		timeZoneName: "longOffset",
	})
		.formatToParts(date)
		.find((p) => p.type === "timeZoneName");
	return part ? part.value.replace("GMT", "UTC") : "UTC";
}

function prettyZone(zone) {
	return zone.replaceAll("_", " ");
}

function renderClocks() {
	const now = new Date();

	if (accountZone) {
		$("clock-time").textContent = formatTime(accountZone, now);
		$("clock-meta").textContent =
			`${formatDate(accountZone, now)} · ${formatOffset(accountZone, now)}`;
	}

	if (lookupZone) {
		$("lookup-time").textContent = formatTime(lookupZone, now);
		$("lookup-meta").textContent =
			`${prettyZone(lookupZone)} · ${formatDate(lookupZone, now)} · ${formatOffset(lookupZone, now)}`;
	}
}

function scheduleClockTick() {
	setTimeout(() => {
		renderClocks();
		scheduleClockTick();
	}, 60000 - (Date.now() % 60000));
}

function renderFormatToggle() {
	$("format-toggle").textContent = prefer24 ? "Switch to 12-hour" : "Switch to 24-hour";
}

function setStatus(message, isError = false) {
	$("status-msg").textContent = message;
	$("status-msg").classList.toggle("is-error", isError);
}

function getJson(url, init) {
	return fetch(url, init)
		.then((res) => (res.ok ? res.json() : null))
		.catch(() => null);
}

async function failureMessage(res, fallback) {
	const body = await res.json().catch(() => ({}));
	return body.message || fallback;
}

function showAccountZone(zone) {
	accountZone = zone;
	$("clock").classList.toggle("hidden", !zone);
	$("delete-timezone").classList.toggle("hidden", !zone);
	$("account-sub").textContent = zone ? prettyZone(zone) : "No timezone set yet";
	renderClocks();
}

function loaded(element) {
	return new Promise((resolve, reject) => {
		element.onload = resolve;
		element.onerror = reject;
	});
}

async function loadPicker() {
	const css = document.createElement("link");
	css.rel = "stylesheet";
	css.href = `${TOM_SELECT}/css/tom-select.css`;
	const script = document.createElement("script");
	script.src = `${TOM_SELECT}/js/tom-select.complete.min.js`;

	const ready = Promise.all([loaded(css), loaded(script)]);
	document.head.prepend(css);
	document.head.append(script);
	await ready;

	ts = new TomSelect("#timezone-select", { maxOptions: 1000 });
}

async function initPicker(timezone) {
	const zones = getJson("/v1/timezones");

	try {
		await loadPicker();
	} catch {
		setStatus("The timezone picker failed to load.", true);
		return;
	}

	if (timezone) {
		ts.addOption({ value: timezone, text: timezone });
		ts.setValue(timezone, true);
	}

	const list = await zones;
	if (!list) {
		setStatus("Failed to load the timezone list.", true);
		return;
	}
	ts.addOptions(list.map((zone) => ({ value: zone, text: zone })));
}

async function loadStats() {
	const stats = await getJson("/v1/stats");

	$("stats-section").classList.toggle("hidden", !stats);
	if (!stats) return;

	$("stat-users").textContent = stats.total_users.toLocaleString();
	$("stat-zones").textContent = stats.unique_timezones.toLocaleString();
	$("stat-week").textContent = stats.recent_registrations.toLocaleString();
	$("stat-top").textContent = stats.top_timezone ? prettyZone(stats.top_timezone) : "None yet";
}

function showSignedIn({ user, timezone }) {
	$("account-name").textContent = user.username;
	if (user.avatar) {
		$("avatar").src =
			`https://cdn.discordapp.com/avatars/${encodeURIComponent(user.id)}/${encodeURIComponent(user.avatar)}.png?size=96`;
		$("avatar").classList.remove("hidden");
	}

	document.body.classList.add("signed-in");
	showAccountZone(timezone);
	initPicker(timezone);
}

async function loadAccount() {
	const data = await getJson("/v1/me", { credentials: "include" });

	if (data) showSignedIn(data);
	document.body.classList.remove("auth-pending");
}

$("use-browser").addEventListener("click", () => {
	if (!ts) return;
	ts.addOption({ value: browserZone, text: browserZone });
	ts.setValue(browserZone);
});

$("format-toggle").addEventListener("click", () => {
	writeFormatPreference(!prefer24);
	renderFormatToggle();
	renderClocks();
});

$("save-timezone").addEventListener("click", async () => {
	const zone = ts?.getValue();
	if (!zone) {
		setStatus("Pick a timezone first.", true);
		return;
	}

	const button = $("save-timezone");
	button.disabled = true;
	setStatus("Saving...");

	try {
		const res = await fetch("/v1/set", {
			method: "POST",
			credentials: "include",
			headers: { "Content-Type": "application/x-www-form-urlencoded" },
			body: new URLSearchParams({ timezone: zone }),
		});
		if (!res.ok) throw new Error(await failureMessage(res, "Could not save your timezone."));
		showAccountZone(zone);
		setStatus("Saved.");
		loadStats();
	} catch (error) {
		setStatus(error.message, true);
	} finally {
		button.disabled = false;
	}
});

$("delete-timezone").addEventListener("click", async () => {
	const button = $("delete-timezone");
	button.disabled = true;

	try {
		const res = await fetch("/v1/delete", { method: "DELETE", credentials: "include" });
		if (!res.ok) throw new Error(await failureMessage(res, "Could not delete your timezone."));
		ts?.clear(true);
		showAccountZone(null);
		setStatus("Timezone deleted.");
		loadStats();
	} catch (error) {
		setStatus(error.message, true);
	} finally {
		button.disabled = false;
	}
});

$("logout").addEventListener("click", async () => {
	try {
		await fetch("/v1/logout", { credentials: "include" });
	} finally {
		window.location.reload();
	}
});

$("lookup-form").addEventListener("submit", async (event) => {
	event.preventDefault();

	const id = $("lookup-id").value.trim();
	lookupZone = null;
	$("lookup-result").classList.add("hidden");
	$("lookup-error").textContent = "";

	if (!/^\d+$/.test(id)) {
		$("lookup-error").textContent = "Enter a numeric Discord user ID.";
		return;
	}

	try {
		const res = await fetch(`/v1/get?id=${encodeURIComponent(id)}`);
		if (res.status === 404) throw new Error("That user hasn't set a timezone.");
		if (!res.ok) throw new Error("Lookup failed. Try again.");

		const data = await res.json();
		$("lookup-name").textContent = data.user.username;
		lookupZone = data.timezone;
		$("lookup-result").classList.remove("hidden");
		renderClocks();
	} catch (error) {
		$("lookup-error").textContent = error.message;
	}
});

document.addEventListener("visibilitychange", () => {
	if (document.visibilityState === "visible") renderClocks();
});

$("browser-zone").textContent = browserZone;
$("use-browser").title = browserZone;
renderFormatToggle();
scheduleClockTick();
loadStats();
loadAccount();
