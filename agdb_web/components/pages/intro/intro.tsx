import styles from "./intro.module.scss";

export const Intro = () => {
    return (
        <div className={styles.intro}>
            <h1>agdb</h1>
            <p className={styles.subheading}>First programming native database without compromises.</p>
            <p className={styles.subheading}><a href="/whyagdb#no-query-language">no query language</a> | <a href="/whyagdb#performance">performance independnet of data size</a> | <a href="/whyagdb#graph">10X cost reduction</a></p>
            <p>
                <a href="/docs/guides/quickstart">Quickstart</a>
                <a href="/blog/blog">Why agdb?</a>
                <a href="/consultation">Expert consultation</a>
            </p>
        </div>
    );
};
