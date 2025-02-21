class TimeclockRequestError extends Error {
	name: string = "TimeclockRequestError"
	constructor(message?: string) {
		let fmt = "could not fetch resource"
		if (!!message) fmt += `: ${message}`
		super(fmt)
	}
}

export interface ClockSchema {
	identity_pool_user_id: string,
	uuid: string,
	name: string,
	last_edit: Date,
	active: boolean,
	clock_in_time: Date | undefined,
}

namespace ServerOutputFix {
	export function clockSchemaInPlace(object: object): asserts object is ClockSchema {
		if (typeof object !== 'object') throw TypeError()
		if (!('identity_pool_user_id' in object)) throw TypeError()
		if (!('uuid' in object)) throw TypeError()
		if (!('name' in object)) throw TypeError()
		if (!('last_edit' in object)) throw TypeError()
		if (!('active' in object)) throw TypeError()
		if (!('clock_in_time' in object)) throw TypeError()

		if (typeof object.last_edit !== "number") throw TypeError()
		object.last_edit = new Date(object.last_edit * 1000)

		if (typeof object.clock_in_time === "number") {
			object.clock_in_time = new Date(object.clock_in_time * 1000)
		} else if (object.clock_in_time !== null) throw TypeError()

		if (typeof object.identity_pool_user_id !== "string") throw TypeError()
		if (typeof object.uuid !== "string") throw TypeError()
		if (typeof object.name !== "string") throw TypeError()
		if (typeof object.active !== "boolean") throw TypeError()
	}
}

export async function loadAllUserClocks(options: { userPoolId: string }): Promise<ClockSchema[]> {
	const url = `http://localhost:4000/clocks/${options.userPoolId}`

	const response = await fetch(url, {
		credentials: 'include'
	});

	if (!response.ok) {
		console.error(response);
		throw new TimeclockRequestError()
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
	const url = `http://localhost:4000/clocks/${options.userPoolId}`

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
	});

	if (!response.ok) {
		console.error(response);
		throw new TimeclockRequestError()
	}

	const deserialized = await response.json();

	ServerOutputFix.clockSchemaInPlace(deserialized);

	return deserialized;
}
