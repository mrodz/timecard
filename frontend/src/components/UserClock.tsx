import React, { forwardRef, PropsWithChildren, useImperativeHandle, useRef, useState } from "react";
import { Card, CardContent, CardDescription, CardTitle } from "./ui/card";
import { ClockSchema } from "@/lib/api";
import useDateFormat from "@/lib/useDateFormat";
import EditClock from "@/components/modals/EditClock";
import { Popover, PopoverContent, PopoverTrigger } from "@/components/ui/popover";
import { Menu } from "lucide-react";
import { Spinner } from "@/components/ui/spinner";

export type UserClockProps = {
	clock: ClockSchema,
	onEdit?: (newClock: ClockSchema) => void,
} | {
	skeleton: string,
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

type UserClockPopoverProps = {
	clock: ClockSchema,
	onClockEdit: (clock: ClockSchema | null) => void;
}

const UserClockPopover: React.FC<UserClockPopoverProps> = (props) => {
	return (
		<Popover>
			<PopoverTrigger className="bg-zinc-100 p-1"><Menu /></PopoverTrigger>
			<PopoverContent>
				<ul className="grid grid-cols-1 items-center">
					<EditClock clock={props.clock} onClockEdit={props.onClockEdit} />
					{/* TODO: Delete Clock */}
				</ul>
			</PopoverContent>
		</Popover>
	)
}


type UserClockRef = {
	edit(clockReplacement: ClockSchema | null): void,
}

const UserClock = forwardRef<UserClockRef, UserClockProps>((props, ref) => {
	if ("skeleton" in props) {
		return (
			<>
				<CardDescription>Last Edit: Just Now</CardDescription>

				<CardContent>
					Loading...
				</CardContent>
			</>
		)
	}

	const [clock, setClock] = useState(props.clock);

	const { formatter } = useDateFormat()

	useImperativeHandle(ref, () => ({
		edit(clockReplacement: ClockSchema | null) {
			if (!!clockReplacement) setClock(clockReplacement)
		}
	}), [setClock])

	let clockIn;

	if (clock.clock_in_time instanceof Date) {
		const maybeClockIn = clock.clock_in_time;
		if (isNaN(maybeClockIn.valueOf())) throw new InvalidUserClockError(InvalidUserClockErrorVariant.ParseClockInDate)
		if (maybeClockIn.valueOf() !== 0) clockIn = maybeClockIn;
	}

	return (
		<>
			<CardDescription>Last Edit: {formatter.date.format(clock.last_edit)}</CardDescription>

			<CardContent>
				{clockIn === undefined ? (
					<div>Not Clocked In</div>
				) : (
					<div>
						{formatter.minute.format(clockIn)}
					</div>
				)}
			</CardContent>

		</>
	)
})

export default (props: UserClockProps) => {
	const clockRef = useRef<UserClockRef>(null)

	const [infallibleName, setInfallibleName] = useState('skeleton' in props ? props.skeleton : props.clock.name)

	const onClockEdit = (newClock: ClockSchema | null) => {
		clockRef.current!.edit(newClock)
		if (!!newClock?.name) {
			setInfallibleName(newClock.name)
		}

		if ('onEdit' in props && !!newClock) {
			props.onEdit?.(newClock)
		}
	}

	return (
		<Card className="w-full p-4">
			<CardTitle>
				<div className="grid grid-cols-[1fr_auto]">
					<span className="truncate">
						{infallibleName}
					</span>
					{'skeleton' in props ? <Spinner /> : <UserClockPopover onClockEdit={onClockEdit} clock={props.clock} />}
				</div>
			</CardTitle>
			<UserClockStack> { /* BEGIN FALLIBLE RENDERING */}
				<UserClock ref={clockRef} {...props} />
			</UserClockStack> { /* END FALLIBLE RENDERING */}
		</Card>
	)
}