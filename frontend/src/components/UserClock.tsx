import React, { PropsWithChildren } from "react";
import { Card, CardContent, CardDescription, CardTitle } from "./ui/card";
import { ClockSchema } from "@/lib/api";

export type UserClockProps = {
	clock: ClockSchema
}

enum InvalidUserClockErrorVariant {
	ParseClockInDate = "The clock in date is not valid",
}

export class InvalidUserClockError extends Error {
	name: string = "InvalidUserClockError";
	constructor(variant: InvalidUserClockErrorVariant) {
		super(variant)
	}
}

class UserClockStack extends React.Component<PropsWithChildren<{}>, { hasError: boolean, error: any }> {
	constructor(props: PropsWithChildren<{}>) {
		super(props)
		this.state = { error: undefined, hasError: false }
	}

	static getDerivedStateFromError(error: any) {
		return { hasError: true, error };
	}

	render() {
		if (this.state.hasError) {
			console.trace((this.state?.error as any) instanceof InvalidUserClockError);
			console.trace(this.state?.error);
			return (
				<div className="bg-red-400 m-4 p-4 flex flex-col justify-center items-center">
					{!!this.state.error && this.state?.error as any instanceof InvalidUserClockError ? (
						<>Error: {this.state.error.message}</>
					) : (
						<pre className="overflow-x-scroll">
							{this.state.error?.trace ?? JSON.stringify(this.state.error)}
						</pre>
					)}
				</div>
			)
		}

		return this.props.children
	}
}

const UserClock: React.FC<UserClockProps> = ({ clock }) => {
	let clockIn;

	if (clock.clock_in_time !== undefined) {
		clockIn = new Date(clock.clock_in_time)
		if (isNaN(clockIn.valueOf())) throw new InvalidUserClockError(InvalidUserClockErrorVariant.ParseClockInDate)
	}

	return (
		<>
			<CardDescription>Last Edit: {clock.last_edit}</CardDescription>

			<CardContent>
				{clockIn === undefined ? (
					<div>Not Clocked In</div>
				) : (
					<div>
						{clockIn.toUTCString()}
					</div>
				)}
			</CardContent>

		</>
	)
}

export default (props: UserClockProps) => {
	return (
		<Card className="w-1/6 p-4">
			<CardTitle>{props.clock.name}</CardTitle>
			<UserClockStack> { /* BEGIN FALLIBLE RENDERING */}
				<UserClock {...props} />
			</UserClockStack> { /* END FALLIBLE RENDERING */}
		</Card>
	)
}