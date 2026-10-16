import { describe, expect, it } from "vitest";
import { emptyAccounts, validateAccounts, validPlayerName } from "./accounts";
describe("verified account selection",()=>{
    const account={id:"a".repeat(32),name:"Alex"};
    it("accepts empty and selected verified libraries",()=>{
        expect(()=>validateAccounts(emptyAccounts())).not.toThrow();
        expect(()=>validateAccounts({revision:2,microsoft:[account],selected:"@"+account.id})).not.toThrow();
    });
    it("rejects unverified, duplicate and dangling identities",()=>{
        for(const value of [
            {revision:0,microsoft:[],selected:"Steve"},
            {revision:0,microsoft:[account,account],selected:"@"+account.id},
            {revision:0,microsoft:[account],selected:null},
            {revision:0,microsoft:[account],selected:"@missing"},
        ])expect(()=>validateAccounts(value)).toThrow();
    });
    it("validates profile names without changing identity",()=>{
        for(const name of ["a","abcdefghijklmnopq"," Steve","A/B"])expect(validPlayerName(name)).toBe(false);
        expect(validPlayerName("Player_123")).toBe(true);
    });
});

it("legacy local selections cannot authorize a game launch",()=>{
    const data={revision:1,names:["Steve"],selected:"Steve"};
    expect(()=>validateAccounts(data)).toThrow();
});
