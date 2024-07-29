import Image from "next/image";

export default function Logo() {
    return (
        <>
            <Image
                src="/images/logo.svg"
                alt="logo"
                width="70"
                height="70"
                style={{ width: "3rem", height: "3rem" }}
            />
            <span style={{ marginLeft: ".8rem" }}>agdb</span>
        </>
    );
}
