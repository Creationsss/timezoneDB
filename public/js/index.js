const loginSection = document.getElementById("login-section");
const timezoneSection = document.getElementById("timezone-section");
const avatarEl = document.getElementById("avatar");
const authStatusEl = document.getElementById("auth-status");
const timezoneSelect = document.getElementById("timezone-select");
const setBtn = document.getElementById("set-timezone");
const statusMsg = document.getElementById("status-msg");

const timezones = Intl.supportedValuesOf("timeZone");
timezones.forEach(tz => {
	const opt = document.createElement("option");
	opt.value = tz;
	opt.textContent = tz;
	timezoneSelect.appendChild(opt);
});

const ts = new TomSelect("#timezone-select", {
	create: false,
	sorted: true,
	searchField: ["text"],
	maxOptions: 1000
});

async function fetchUserInfo() {
	try {
		const res = await fetch("/me", { credentials: "include" });
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

		if (tz) {
			ts.setValue(tz);
			deleteBtn.classList.remove("hidden");
		} else {
			ts.clear();
			deleteBtn.classList.add("hidden");
		}

		deleteBtn.addEventListener("click", async () => {
			try {
				const res = await fetch("/delete", { credentials: "include" });
				if (!res.ok) throw new Error();

				ts.clear();
				statusMsg.textContent = "Timezone deleted.";
				deleteBtn.classList.add("hidden");
			} catch {
				statusMsg.textContent = "Failed to delete timezone.";
			}
		});

	} catch {
		loginSection.classList.remove("hidden");
		timezoneSection.classList.add("hidden");
	}
}

setBtn.addEventListener("click", async () => {
	const timezone = ts.getValue();
	if (!timezone) return;

	try {
		const res = await fetch(`/set?timezone=${encodeURIComponent(timezone)}`, {
			credentials: "include",
		});
		if (!res.ok) throw new Error();
		statusMsg.textContent = "Timezone updated!";
	} catch {
		statusMsg.textContent = "Failed to update timezone.";
	}
});

fetchUserInfo();
