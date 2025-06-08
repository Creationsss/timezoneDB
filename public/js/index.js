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
				const res = await fetch("/delete", {
					method: "DELETE",
					credentials: "include",
				});

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

	setBtn.disabled = true;
	setBtn.textContent = "Saving...";
	statusMsg.textContent = "";

	try {
		const formData = new FormData();
		formData.append("timezone", timezone);

		const res = await fetch("/set", {
			method: "POST",
			credentials: "include",
			body: formData,
		});

		if (!res.ok) {
			const error = await res.json();
			throw new Error(error.message || "Failed to update timezone");
		}

		statusMsg.textContent = "Timezone updated!";
		document.getElementById("delete-timezone").classList.remove("hidden");
	} catch (error) {
		statusMsg.textContent = error.message;
	} finally {
		setBtn.disabled = false;
		setBtn.textContent = "Save";
	}
});

fetchUserInfo();
