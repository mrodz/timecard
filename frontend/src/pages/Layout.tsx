import { Component, createContext, Suspense, use, useCallback, useEffect, useRef, useState } from 'react';
import { Link, Outlet, useNavigate } from 'react-router-dom';
import useAuth from "@/lib/useAuth"
import AppHeader from '@/components/AppHeader';
import { Spinner } from '@/components/ui/spinner';
import { CognitoUser, CognitoUserAttribute } from 'amazon-cognito-identity-js';
import { Button, buttonVariants } from '@/components/ui/button';

type DashboardLoadedProps = {
	currentUser: Promise<CognitoUser | null>,
	signOut(): Promise<void>,
}

type UserContext = {
	reactiveUser: CognitoUser | null,
	attributes: CognitoUserAttribute[] | undefined;
	signOut(): Promise<void>,
}

export const CurrentUserContext = createContext<UserContext | null>({
	reactiveUser: null,
	attributes: undefined,
	async signOut() { }
})

export function ProtectedRouteLoadingScreenReject(props: { message: string, willRedirect?: string, delay?: number }) {
	const [count, setCount] = useState(Math.ceil(props.delay ?? 0))
	const navigate = useNavigate()

	useEffect(() => {
		let timeout: NodeJS.Timeout;
		if (!!props.willRedirect) {
			timeout = setInterval(() => {
				setCount(count => count - 1)
			}, 1_000)
		}
		return () => !!timeout && clearInterval(timeout)
	})

	return (
		<div className='bg-red-400 w-full min-h-screen flex items-center justify-center'>
			<div className='flex flex-col items-center'>
				<h1 className='mb-8'>
					{props.message}
				</h1>

				{props.willRedirect ? (
					<div>
						<p>
							You will be redirected in {count} seconds. Click below to log in <u>now</u>
						</p>

						<Link className={buttonVariants({ variant: 'link' })} to={props.willRedirect}>Go to sign in</Link>
					</div>
				) : (
					<div>
						<Button variant='default' onClick={() => navigate(-1)}>Go Back</Button>
					</div>
				)}
			</div>
		</div>
	)
}

function LayoutLoaded(props: DashboardLoadedProps) {
	const user: CognitoUser | null = use(props.currentUser)

	const [reactiveUser, setReactiveUser] = useState<typeof user>(user);
	const [attributes, setAttributes] = useState<CognitoUserAttribute[] | undefined>(undefined);
	const [willRedirect, setWillRedirect] = useState<string | undefined>();

	const notSignedInActionLogin = useRef(true);


	useEffect(() => {
		let timeout: NodeJS.Timeout

		// not signed in
		if (reactiveUser === null) {
			const params = new URLSearchParams()
			params.set('client_id', import.meta.env.VITE_COGNITO_CLIENT_ID)
			params.set('response_type', 'code')
			params.set('redirect_uri', 'http://localhost:5173/auth/')

			if (notSignedInActionLogin.current) {
				const urlCopy = `https://auth.timecard.pro/login?${params.toString()}&scope=aws.cognito.signin.user.admin+email+openid+phone`
				setWillRedirect(urlCopy)

				timeout = setTimeout(() => {
					window.location.href = urlCopy
				}, 7_000)
			}
		} else {
			reactiveUser.getUserAttributes((err: Error | undefined, ok: CognitoUserAttribute[] | undefined) => {
				if (err) {
					console.error(err);
				} else {
					setAttributes(ok)
				}
			})

		}

		return () => !!timeout && clearTimeout(timeout)
	}, [reactiveUser])

	const signOut = async () => {
		await props.signOut()
		notSignedInActionLogin.current = false;
		setReactiveUser(null)
	}

	return (
		user === null
			? <ProtectedRouteLoadingScreenReject message="You aren't signed in and can't access this resource" willRedirect={willRedirect} delay={7} />
			: (
				<CurrentUserContext.Provider value={{ reactiveUser, attributes, signOut }}>
					<div className='w-full flex flex-col'>
						<AppHeader />
						<Outlet />
					</div>
				</CurrentUserContext.Provider>
			)
	)
}

export function ProtectedRouteLoadingScreenSuccess(props: { message: string }) {
	return (
		<div className='bg-lime-500 w-full min-h-screen flex items-center justify-center'>
			<div className='flex flex-col items-center'>
				<h1 className='mb-8'>
					{props.message}
				</h1>
				<Spinner />
			</div>
		</div>
	)
}

export default function Layout() {
	const { getCurrentUser, signOut } = useAuth()

	return (
		<Suspense fallback={<ProtectedRouteLoadingScreenSuccess message="Application is loading" />}>
			<LayoutLoaded currentUser={getCurrentUser()} signOut={signOut} />
		</Suspense>
	)
}