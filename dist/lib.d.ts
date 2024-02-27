export interface Payload {
    pdf_name: string;
    transactions: Transaction[];
}
export declare function generateStatement(payload: string, mmf: boolean): Promise<void>;
