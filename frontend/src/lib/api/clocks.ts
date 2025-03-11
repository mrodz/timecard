import { ClockSchema, ServerOutputFix, TimeclockRequestError } from "../api";

export async function loadAllUserClocks(options: { userPoolId: string }): Promise<ClockSchema[]> {
	const url = `http://localhost:4000/user/${options.userPoolId}/clocks`

	const response = await fetch(url, {
		credentials: 'include'
	})

	if (!response.ok) {
		console.error(response);
		throw new TimeclockRequestError(await response.text())
	}

	const deserialized = await response.json();

	if (!Array.isArray(deserialized)) {
		throw new TimeclockRequestError("did not produce array")
	}

	for (let i = 0; i < deserialized.length; i++) {
		ServerOutputFix.clockSchemaInPlace(deserialized[i]);
	}

	return deserialized;
}

export async function createUserClock(options: { userPoolId: string, name: string }): Promise<ClockSchema> {
	const url = `http://localhost:4000/user/${options.userPoolId}/clocks`

	const response = await fetch(url, {
		credentials: 'include',
		method: 'POST',
		headers: {
			'Accept': 'application/json',
			'Content-Type': 'application/json'
		},
		body: JSON.stringify({
			name: options.name,
		})
	})

	if (!response.ok) {
		console.error(response);
		throw new TimeclockRequestError(await response.text())
	}

	const deserialized = await response.json();

	ServerOutputFix.clockSchemaInPlace(deserialized);

	return deserialized;
}

export async function editUserClock(options: {
	userPoolId: string,
	clockUuid: string,
	edits: {
		name?: string,
	}
}): Promise<ClockSchema | null> {
	const url = `http://localhost:4000/user/${options.userPoolId}/clocks/${options.clockUuid}/edit`

	const response = await fetch(url, {
		credentials: 'include',
		method: 'POST',
		headers: {
			'Accept': 'application/json',
			'Content-Type': 'application/json'
		},
		body: JSON.stringify({
			name: options.edits?.name,
		})
	})

	if (!response.ok) {
		console.error(response);
		throw new TimeclockRequestError(await response.text())
	}

	const deserialized = await response.json();

	if (typeof deserialized !== 'object' || !("clock" in deserialized)) {
		throw TypeError("malformed return value: " + JSON.stringify(deserialized))
	}

	const { clock } = deserialized;

	if (clock === null) return null;

	ServerOutputFix.clockSchemaInPlace(clock);

	return clock;
}

export async function deleteUserClock(options: {
	userPoolId: string,
	clockUuid: string,
}): Promise<ClockSchema | null> {
	const url = `http://localhost:4000/user/${options.userPoolId}/clocks/${options.clockUuid}/delete`

	const response = await fetch(url, {
		credentials: 'include',
		method: 'POST',
		headers: {
			'Accept': 'application/json',
			'Content-Type': 'application/json'
		},
	})

	if (!response.ok) {
		console.error(response);
		throw new TimeclockRequestError(await response.text())
	}

	const deserialized = await response.json();

	if (typeof deserialized !== 'object' || !("clock" in deserialized)) {
		throw TypeError("malformed return value: " + JSON.stringify(deserialized))
	}

	const { clock } = deserialized;

	if (clock === null) return null;

	ServerOutputFix.clockSchemaInPlace(clock);

	return clock;
}
