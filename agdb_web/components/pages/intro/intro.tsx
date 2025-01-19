import styles from "./intro.module.scss";

export const Intro = () => {
    return (
        <div className={styles.intro}>
            <h1>agdb</h1>
            <p className={styles.subheading}>
                First application native database without compromises.
            </p>
            <p className={styles.subheading}>
                no query language | performance independnet of data size | 10X
                cost reduction
            </p>
            <p>
                <a href="/docs/guides/quickstart">Quickstart</a>
                <a href="/whyagdb">Why agdb?</a>
                <a href="/enterprise/consultation">Expert consultation</a>
            </p>
        </div>
    );
};
