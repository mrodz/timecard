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
	last_edit: string,
	active: boolean,
	clock_in_time: string | undefined,
}

export async function loadAllUserClocks(options: { userPoolId: string }): Promise<ClockSchema[]> {
	const url = `http://localhost:4000/clocks/${options.userPoolId}`

	const response = await fetch(url, {
		credentials: 'include'
	});

	if (!response.ok) {
		throw new TimeclockRequestError()
	}

	const deserialized = await response.json();

	if (!Array.isArray(deserialized)) {
		throw new TimeclockRequestError("did not produce array")
	}

	return deserialized;
}