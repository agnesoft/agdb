import { addBasePath } from "next/dist/client/add-base-path";
import { NextResponse } from "next/server";
import type { NextRequest } from "next/server";

const locales = JSON.parse(process.env.NEXTRA_LOCALES!) as string[];

const defaultLocale = process.env.NEXTRA_DEFAULT_LOCALE!;
const HAS_LOCALE_RE = new RegExp(`^\\/(${locales.join("|")})(\\/|$)`);
const COOKIE_NAME = "NEXT_LOCALE";

export function middleware(request: NextRequest) {
    const { pathname } = request.nextUrl;

    // Check if there is any supported locale in the pathname
    const pathnameHasLocale = HAS_LOCALE_RE.test(pathname);
    const cookieLocale = request.cookies.get(COOKIE_NAME)?.value;

    // Redirect if there is no locale
    if (!pathnameHasLocale) {
        const locale = cookieLocale || defaultLocale;

        const url = addBasePath(`/${locale}${pathname}`);
        return NextResponse.redirect(new URL(url, request.url));
    }

    const requestLocale = pathname.split("/", 2)[1];

    if (requestLocale !== cookieLocale) {
        const response = NextResponse.next();
        response.cookies.set(COOKIE_NAME, requestLocale);
        return response;
    }
}

export const config = {
    matcher: ["/((?!_next/static|_next/image|favicon.ico|images|manifest).*)"],
};
