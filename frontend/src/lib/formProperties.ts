import { z } from "zod";

export namespace Clock {
	export const NAME = z.string({
		required_error: "Name is required",
	}).trim().min(1, {
		message: 'Name must not be empty',
	}).max(64, {
		message: 'Name is too long'
	})
}