# Rust Data API

# Endpoints

## Insert One

### Options

```ts
type InsertOneOptions = {
  bypassDocumentValidation: boolean;
  comment: string;
  writeConcern: {
    j: true;
    w: "majority" | number | "custom tags";
    wtimeout: number;
  };
};
```
