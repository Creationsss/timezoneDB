const loginSection = document.getElementById("login-section");
const timezoneSection = document.getElementById("timezone-section");
const avatarEl = document.getElementById("avatar");
const authStatusEl = document.getElementById("auth-status");
const timezoneSelect = document.getElementById("timezone-select");
const setBtn = document.getElementById("set-timezone");
const statusMsg = document.getElementById("status-msg");

const timezones = Intl.supportedValuesOf("timeZone");

for (const tz of timezones) {
	const opt = document.createElement("option");
	opt.value = tz;
	opt.textContent = tz;
	timezoneSelect.appendChild(opt);
}

const ts = new TomSelect("#timezone-select", {
	create: false,
	sorted: true,
	searchField: ["text"],
	maxOptions: 1000,
});

let timeInterval;
let userPreferred24Hour = null;

function detectBrowserTimeFormat() {
	const testFormatter = new Intl.DateTimeFormat(navigator.language, {
		hour: "numeric",
		minute: "2-digit",
	});
	const testTime = testFormatter.format(new Date("2023-01-01 13:00:00"));
	return !(
		testTime.includes("PM") ||
		testTime.includes("AM") ||
		testTime.includes("pm") ||
		testTime.includes("am")
	);
}

function getCurrentTimeFormat() {
	if (userPreferred24Hour === null) {
		return detectBrowserTimeFormat();
	}
	return userPreferred24Hour;
}

function saveTimeFormatPreference(is24Hour) {
	userPreferred24Hour = is24Hour;
	localStorage.setItem("timeFormat24Hour", is24Hour.toString());
}

function loadTimeFormatPreference() {
	const saved = localStorage.getItem("timeFormat24Hour");
	if (saved !== null) {
		userPreferred24Hour = saved === "true";
	}
}

function createTimeDisplay() {
	if (document.getElementById("time-display")) return;

	const timeContainer = document.createElement("div");
	timeContainer.id = "time-display";
	timeContainer.className = "time-display hidden";
	timeContainer.innerHTML = `
        <div class="current-time">
            <h3>Your Current Time</h3>
            <div id="current-time-value">--:--:--</div>
            <div id="current-date-value">----</div>
        </div>
    `;

	const profileSection = document.querySelector(".profile");
	profileSection.after(timeContainer);
}

function updateTime(timezone) {
	if (!timezone) return;

	const timeDisplay = document.getElementById("time-display");
	const timeValue = document.getElementById("current-time-value");
	const dateValue = document.getElementById("current-date-value");

	if (!timeDisplay || !timeValue || !dateValue) return;

	const now = new Date();
	const is24Hour = getCurrentTimeFormat();

	const formatter = new Intl.DateTimeFormat(navigator.language, {
		timeZone: timezone,
		hour: "2-digit",
		minute: "2-digit",
		second: "2-digit",
		hour12: !is24Hour,
	});

	const dateFormatter = new Intl.DateTimeFormat(navigator.language, {
		timeZone: timezone,
		weekday: "long",
		year: "numeric",
		month: "long",
		day: "numeric",
	});

	timeValue.textContent = formatter.format(now);
	dateValue.textContent = dateFormatter.format(now);
	timeDisplay.classList.remove("hidden");
}

function updateTimezoneInfo(timezone) {
	if (!timezone) return;

	const now = new Date();
	const offsetFormatter = new Intl.DateTimeFormat(navigator.language, {
		timeZone: timezone,
		timeZoneName: "longOffset",
	});

	const offsetParts = offsetFormatter.formatToParts(now);
	const offset =
		offsetParts.find((part) => part.type === "timeZoneName")?.value || "UTC";

	const infoSection = document.getElementById("timezone-info");
	const infoCards = document.getElementById("info-cards");
	const offsetEl = document.getElementById("timezone-offset");
	const descriptionEl = document.getElementById("timezone-description");
	const utcOffsetEl = document.getElementById("utc-offset");
	const timeFormatEl = document.getElementById("time-format");

	if (infoSection && offsetEl && descriptionEl) {
		offsetEl.textContent = offset;
		descriptionEl.textContent = timezone.replace(/_/g, " ");
		utcOffsetEl.textContent = offset;

		const is24Hour = getCurrentTimeFormat();
		timeFormatEl.textContent = is24Hour ? "24h" : "12h";
		timeFormatEl.style.cursor = "pointer";
		timeFormatEl.title = "Click to toggle between 12h and 24h format";

		timeFormatEl.onclick = () => {
			const newFormat = !getCurrentTimeFormat();
			saveTimeFormatPreference(newFormat);
			updateTimezoneInfo(timezone);
			updateTime(timezone);
		};

		infoSection.classList.remove("hidden");
		infoCards.classList.remove("hidden");
	}
}

async function fetchStats() {
	try {
		const response = await fetch("/v1/stats", { credentials: "include" });
		if (!response.ok) throw new Error();

		const json = await response.json();

		const section = document.getElementById("stats-section");
		const statsElements = {
			totalUsers: document.getElementById("stat-total-users"),
			uniqueTimezones: document.getElementById("stat-unique-timezones"),
			recentUsers: document.getElementById("stat-recent-users"),
			topTimezones: document.getElementById("stat-top-timezone"),
		};

		if (json) {
			statsElements.totalUsers.textContent = json.total_users;
			statsElements.uniqueTimezones.textContent = json.unique_timezones;
			statsElements.recentUsers.textContent = json.recent_registrations;
			statsElements.topTimezones.textContent = json.top_timezone;

			section.classList.remove("hidden");
		}
	} catch (error) {
		console.error(error);
	}
}

async function fetchUserInfo() {
	try {
		const res = await fetch("/v1/me", { credentials: "include" });
		if (!res.ok) throw new Error();

		const json = await res.json();
		const user = json.user;
		const tz = json.timezone;

		authStatusEl.textContent = user.username;

		if (user.avatar) {
			avatarEl.src = `https://cdn.discordapp.com/avatars/${user.id}/${user.avatar}.png`;
			avatarEl.classList.remove("hidden");
		}

		loginSection.classList.add("hidden");
		timezoneSection.classList.remove("hidden");

		const deleteBtn = document.getElementById("delete-timezone");
		const logoutBtn = document.getElementById("logout");

		logoutBtn.classList.remove("hidden");

		if (tz) {
			ts.setValue(tz);
			deleteBtn.classList.remove("hidden");

			if (timeInterval) clearInterval(timeInterval);
			createTimeDisplay();
			updateTime(tz);
			updateTimezoneInfo(tz);
			timeInterval = setInterval(() => updateTime(tz), 1000);
		} else {
			ts.clear();
			deleteBtn.classList.add("hidden");
			if (timeInterval) clearInterval(timeInterval);
			const timeDisplay = document.getElementById("time-display");
			const timezoneInfo = document.getElementById("timezone-info");
			const infoCards = document.getElementById("info-cards");

			if (timeDisplay) timeDisplay.classList.add("hidden");
			if (timezoneInfo) timezoneInfo.classList.add("hidden");
			if (infoCards) infoCards.classList.add("hidden");
		}

		deleteBtn.addEventListener("click", async () => {
			try {
				const res = await fetch("/v1/delete", {
					method: "DELETE",
					credentials: "include",
				});

				if (!res.ok) throw new Error();

				ts.clear();
				statusMsg.textContent = "Timezone deleted.";
				deleteBtn.classList.add("hidden");

				if (timeInterval) clearInterval(timeInterval);
				const timeDisplay = document.getElementById("time-display");
				const timezoneInfo = document.getElementById("timezone-info");
				const infoCards = document.getElementById("info-cards");

				if (timeDisplay) timeDisplay.classList.add("hidden");
				if (timezoneInfo) timezoneInfo.classList.add("hidden");
				if (infoCards) infoCards.classList.add("hidden");
			} catch {
				statusMsg.textContent = "Failed to delete timezone.";
			}
		});

		logoutBtn.addEventListener("click", async () => {
			try {
				const res = await fetch("/v1/logout", {
					method: "GET",
					credentials: "include",
				});

				if (!res.ok) throw new Error();

				statusMsg.textContent = "Logged out.";
				loginSection.classList.remove("hidden");
				timezoneSection.classList.add("hidden");
			} catch {
				statusMsg.textContent = "Failed to log out.";
			}
		});
	} catch {
		loginSection.classList.remove("hidden");
		timezoneSection.classList.add("hidden");

		try {
			await fetchStats();
		} catch (error) {
			console.error(error);
		}
	}
}

setBtn.addEventListener("click", async () => {
	const timezone = ts.getValue();
	if (!timezone) return;

	setBtn.disabled = true;
	setBtn.textContent = "Saving...";
	statusMsg.textContent = "";

	try {
		const params = new URLSearchParams();
		params.append("timezone", timezone);

		const res = await fetch("/v1/set", {
			method: "POST",
			credentials: "include",
			headers: {
				"Content-Type": "application/x-www-form-urlencoded",
			},
			body: params,
		});

		if (!res.ok) {
			const error = await res.json();
			throw new Error(error.message || "Failed to update timezone");
		}

		statusMsg.textContent = "Timezone updated!";
		document.getElementById("delete-timezone").classList.remove("hidden");

		if (timeInterval) clearInterval(timeInterval);
		createTimeDisplay();
		updateTime(timezone);
		updateTimezoneInfo(timezone);
		timeInterval = setInterval(() => updateTime(timezone), 1000);
	} catch (error) {
		statusMsg.textContent = error.message;
	} finally {
		setBtn.disabled = false;
		setBtn.textContent = "Save Timezone";
	}
});

loadTimeFormatPreference();

fetchUserInfo();
