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

type ClockSchemaUnfixedDates = Omit<ClockSchema, keyof ['last_edit', 'clock_in_time']> & {
	last_edit: number,
	clock_in_time: number | undefined,
}

function fixDatesInPlaceServerOutput(object: object): asserts object is ClockSchema {
	if ('last_edit' in object && typeof object.last_edit === "number") {
		object.last_edit = new Date(object.last_edit * 1000)
	}

	if ('clock_in_time' in object && typeof object.clock_in_time === "number") {
		object.clock_in_time = new Date(object.clock_in_time * 1000)
	}

	console.log(object)
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
		fixDatesInPlaceServerOutput(deserialized[i]);
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

	fixDatesInPlaceServerOutput(deserialized);

	return deserialized;
}
