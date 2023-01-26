import { FnkBorshWriter } from '../../serializer';
import { FnkBorshReader } from '../../deserializer';
import { InferFnkBorshSchemaInner, RustMap } from '../maps';
import { FnkBorshSchema } from '../../borsh';
import { U16, U8 } from '../unsigned';

export function FnkBVec<
    Sk extends FnkBorshSchema<any>,
    Sv extends FnkBorshSchema<any>
>({ keySchema, valueSchema }: { keySchema: Sk; valueSchema: Sv }) {
    return new FnkBVecSchema(keySchema, valueSchema);
}

export class FnkBVecSchema<
    Sk extends FnkBorshSchema<any>,
    Sv extends FnkBorshSchema<any>
> implements
        FnkBorshSchema<
            RustMap<InferFnkBorshSchemaInner<Sk>, InferFnkBorshSchemaInner<Sv>>
        >
{
    readonly keySchema: Sk;
    readonly valueSchema: Sv;

    // CONSTRUCTOR ------------------------------------------------------------

    constructor(keySchema: Sk, valueSchema: Sv) {
        this.keySchema = keySchema;
        this.valueSchema = valueSchema;
    }

    // METHODS ----------------------------------------------------------------

    /**
     * Values must be sorted by key.
     */
    serialize(
        writer: FnkBorshWriter,
        value: RustMap<
            InferFnkBorshSchemaInner<Sk>,
            InferFnkBorshSchemaInner<Sv>
        >
    ) {
        let nodes: Node<
            InferFnkBorshSchemaInner<Sk>,
            InferFnkBorshSchemaInner<Sv>
        >[] = value.map((v) => {
            return {
                key: v.key,
                value: v.value,
                leftChildAt: 0,
                rightChildAt: 0,
                height: 0,
            };
        });

        let root = fixNode(0, nodes.length, nodes);

        if (!root) {
            U16.serialize(writer, 0);
            U16.serialize(writer, 0);
        } else {
            let [rootPosition, _] = root;

            U16.serialize(writer, nodes.length);
            U16.serialize(writer, rootPosition + 1);

            for (const node of nodes) {
                this.keySchema.serialize(writer, node.key);
                this.valueSchema.serialize(writer, node.value);
                U16.serialize(writer, node.leftChildAt);
                U16.serialize(writer, node.rightChildAt);
                U8.serialize(writer, node.height);
            }
        }
    }

    deserialize(
        reader: FnkBorshReader
    ): RustMap<InferFnkBorshSchemaInner<Sk>, InferFnkBorshSchemaInner<Sv>> {
        const length = U16.deserialize(reader);
        U16.deserialize(reader);

        const result: RustMap<
            InferFnkBorshSchemaInner<Sk>,
            InferFnkBorshSchemaInner<Sv>
        > = [];

        for (let i = 0; i < length; i++) {
            const key = this.keySchema.deserialize(reader);
            const value = this.valueSchema.deserialize(reader);
            U16.deserialize(reader);
            U16.deserialize(reader);
            U8.deserialize(reader);

            result.push({
                key,
                value,
            });
        }

        return result;
    }
}

interface Node<K, V> {
    key: K;
    value: V;
    leftChildAt: number;
    rightChildAt: number;
    height: number;
}

// [from, to)
function fixNode<K, V>(
    from: number,
    to: number,
    nodes: Node<K, V>[]
): [number, Node<K, V>] | null {
    if (from === to) {
        return null;
    }

    const centerPosition = Math.floor((from + to) / 2);
    const node = nodes[centerPosition];
    node.height = 0;

    const leftChild = fixNode(from, centerPosition, nodes);
    const rightChild = fixNode(centerPosition + 1, to, nodes);

    if (leftChild) {
        const [leftChildPosition, leftChildNode] = leftChild;
        node.leftChildAt = leftChildPosition + 1;
        node.height = 1 + leftChildNode.height;
    } else {
        node.leftChildAt = 0;
    }

    if (rightChild) {
        const [rightChildPosition, rightChildNode] = rightChild;
        node.rightChildAt = rightChildPosition + 1;
        node.height = Math.max(node.height, 1 + rightChildNode.height);
    } else {
        node.rightChildAt = 0;
    }

    return [centerPosition, node];
}
