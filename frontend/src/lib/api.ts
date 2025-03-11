export class TimeclockRequestError extends Error {
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

export namespace ServerOutputFix {
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