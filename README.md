# FLUF Protocol - Flash Loan Unlimited Facility on Solana

## Demo

See video [here](./demo/README.md) and on [YouTube here]().

## Abstract

A novel universal Flash Loan Facility is described and prototyped on Solana. With an unusual calling pattern, it allows
for Flash Loans in multiple protocols that are agnostic to and unaware of Flash Loans. In addition,
this facility allows for a novel mechanism in which unlimited non-existent funds can be borrowed for the duration
of a single transaction, to achieve even more powerful results than the typical flash loans.

## Introduction

Flash loans are a new lending mechanism typical to Decentralized Finance (DeFi). Protocols are able to
borrow funds for the duration of a single atomic transaction. The transaction can succeed only if the flash loan is
repaid within the same transaction along with the associated fee. 

Flash loans have allowed arbitrage, timely liquidations and other very useful operations to be performed by parties
with no sizable funds making DeFi a plain field available to anyone and more fair. This availability is making
the associated DeFi protocols more nimble and thus more stable, more liquid and price-adjusted against other markets.

Flash loans have been a tool for abuse as well, but that has generally occurred with protocols that are poorly written
and as such, subject to manipulation of price oracles. In addition, flash loans have been used to amplify exploits in buggy
protocols and maximize the attack effects.

One of the first protocols allowing this mechanism was Uniswap, which called it Flash Swaps:

![Uniswap Flash Swap](./Uniswap%20Flash%20Swap.png)

The purpose of of the
Flash Swaps was to perform price arbitrage and equalize the Uniswap price against other on-chain exchanges. To achieve
this, the sequence of swapping asset A for asset B was:
1. Optimistically receive the asset B (in counter-value of the asset A).
2. Having received asset B, while still in possession of asset A, call out another protocol (intentional callback). 
This would typically be a swap of asset B to A at another decentralized exchange (DEX) at better price.
3. Having received more A because of the other DEX's better price, pay the required counter-value of asset A, and
have some left-over of A to pay the fee and receive profit.

Note that the above Flash Swap can be executed by caller that has no funds (other than for gas), and make profit,
while importantly performing price arbitrage to equalize the appropriate Uniswap market (pool) with the other DEX.
Note that the borrowed funds have to previously exist in the appropriate Uniswap pool before calling the Flash Swap.

There is also an additional issue with Flash Loans: they are not available when they are needed the most. Namely,
in times of low liquidity, the sources of flash lending are insufficient due to market conditions.

Our calling pattern is different, as it originates at our protocol and ends with our protocol, thus allowing for more
universality.

In addition, it allows for **borrowing funds that do not exist**. This technique
has been proven in a typical calling pattern with single protocol and demonstrated in liquidation of a delinquent leveraged position,
otherwise not possible with usual flash loan, at [CFD DEX and Flash Trillions](https://github.com/jordan-public/cfd-dex)
presented at 
[ETHGlobal Scaling Ethereum 2023](https://ethglobal.com/showcase/cfd-dex-and-flash-trillions-ad666). 

However, while such flash loan of non-existent funds was demonstrated to work in a single protocol for liquidations,
**we go one step further**, allowing such Unlimited Flash Loans that borrow non-existent funds, to be used **across multiple protocols**.

Read on...

## Flash Loan Facility

In order to allow for universal Flash Loan Facility usable by sequence of protocols all unaware of the flash loans, we use a different calling pattern / sequence:

![Flash Loan Facility](./Flash%20Loan%20Facility.png)

1. The Flash Loan Facility borrows funds from its pool (PDA - Solana Program Derived Account) and transfers the borrowed funds to the user's program PDA.
2. The Flash Loan Facility calls (via CPI - Solana Cross Program Invocation) the user-written program intended for execution of the Flash Loan transaction.
3. Having funds available, the user-written program permissionlessly calls a sequence of protocols it wants to utilize.
4. Once the call to the user-written program returns, the Flash Loan Facility
checks whether the loan is repaid along with the fee and otherwise it reverts
the entire transaction.

But what does the "U" stand for in the FLUF Protocol? Read on...

## Unlimited Flash Loans - Borrowing Non-existent Funds

**FLUF** stands for **Flash Loan Unlimited Facility**. The FLUF Protocol can offer Flash Loans of **any** requested amount, regardless whether the funds exist or not. How does this work? Here is the explanation:

Protocols are usually agnostic to the assets used and anyone can create new pools. For example, in an AMM (Automated Market Maker) anyone can create a Liquidity Pool for trading any desired pair of tokens A and B. As appropriate
amounts of A and B are deposited in the Liquidity Pool, then anyone can trade
against the Liquidity Pool.

Let's say we wrap both tokens A and B with another pair of tokens wA and wB, in one-to-one proportion (each token wA contains/wraps one token A and each token wB contains/wraps one token B).

We can then create a Liquidity Pool for trading wA and wB and deposit wA and wB in it. This is equivalent in value to the original Liquidity Pool of A and B
as long as the amounts of wA and wB in the new pool are the same as the amounts of A and B in the original pool. To trade A for B in the new pool,
we can wrap A into wA, trade wA for wB and then unwrap B from the returned wB,
thus achieving the same as trading A for B in the original pool.

To recap:
- To **wrap** amount x of A into wA means to lock amount x of A, then to mint
amount x of wA and to return it to the caller.
-  To **unwrap** amount x of wA means to burn amount x of wA, then to unlock
amount x of A and return it to the caller.

### FLUF Protcol Rules

The FLUF Protocol (Flash Loan **Unlimited** Facility Protocol) operates Pools of capital (tokens) and it is in charge of minting and burning but it enforces the following rules:

1. Any **investor** depositing a token **T** into the FLUF Pool, receives an equivalent value of amount of token **fT**. Note that the amount of fT received is not necessary the same amount of T deposited, as fT is an appreciating asset, as we will see that in the "Economics" section below. In addition, as the fT tokens contain actual collateralized value, they can be
used in other DeFi protocols, for additional yield, but that's irrelevant
to our explanation.
2. The FLUF Protocol does not lend the deposited tokens **T**, but instead it lends **ft**.
Outside of Flash Loans each fT is collateralized by T.
3. Outside of Flash Loans, only **T** can be exchanged for **fT** (Pool Deposit) and vice versa (Pool Withdrawal).
4. Inside Flash Loan (entrypoint ```lendAndCall```), 
the FLUF Protocol **mints** **any** requested amount of fT without collateral (regardless of the amount of T in the pool). However, this amount has to be repaid by the end of the transaction along with the fee, but then the FLUF Protocol **burns** only the amount that was lent. The reason why the FLUF Protocol can mint any amount without limit
is because it does so without collateral, yet with appropriate repayment accounting. 

![Uncollateralized Minting](./Uncollateralized%20Minting.png)

### FLUF Protocol Roles

The participants in the FLUF Protocol are the folloiwng:
1. **Investor**: deposits tokens T into the appropriate FLUF Protocol Pool. Each token has its own pool, which is created permissionlessly by anyone, upon its first usage. In exchange for the deposited T the investors gets an equivalent counter-value fT tokens (FLUF-T). fT is an appreciating asset against T, so the investor depositing T to get a certain amount of fT, can withdraw the his investment and receive more T than initially
deposited.
2. **Protocol**: can use fT as any other token. This could be collateral for CFD or Perp 
Decentralized Exchanges (DEX), liquidity pools for lending and/or borrowing fT, spot DEXes
for trading fT etc. This can be done in a permissioned manner by the protocol,
or permissionlessly by the users of protocols that allow it. All such protocols, even
previously unaware of flash loans, can utilize fT Flash Lending to stabilize their
locked capital by price arbitrage, liquidation of delinquent accounts etc.
3. **Flash Loan User**: can borrow any amount of fT to perform above mentioned arbitrage,
liquidations etc. If the Flash Loan User needs T instead of fT, even for the duration of
the Flash Loan (within an atomic transaction), he can convert fT to T up to the existent
amount of T in the pool, and utilize it in protocols that operate on any token T.
More importantly, fT can be borrowed in arbitrary amounts, making the liquidity pools,
lending pools, fT-collateralized protocols, etc. even more stable, by being able to
perform the desired actions even in liquidity crunches.

## Economics

As each Flash Loan has to be repaid (back to the Pool) in token fT along
with the **fee** of **0.25%** (1/400) of the original amount borrowed. 1/5 of that fee goes to the FLUF Protocol and the remaining 5/6 is burned, 
thus distributing the fees to the depositors (Investors) via deflation,
as their unchanged deposit in fT results in more T. To be able to pay the fee in fT, the Flash Loan Borrowers have to deposit more T to receive the needed
fT.

With the above we can see that the pool grows in value, which is the reason why 1 fT is not equal to 1 T. Each pool has a factor $f$. Then the following rules are enforced:
- When an investor deposits into the pool, the factor $f_d = f$ is recorded with the deposit. If there was a previous deposit of $d$ tokens T and depositing new $n$ tokens T, instead of rebasing the total deposit to: $d * f / f_d + n$, the factor $f_d$ is adjusted to achieve equivalence at the same
amount of deposit $d + n$: $f_d = f_d * (d+n) / (d * f / f_d + n)$.
- When a Flash Loan is repaid, it's factor **f** is adjusted as follows: $f = f * T_a / T_b$, where $T_b$ is the amount of T in the pool before the flash loan and $T_a$ is the amount of T in the pool after the flash loan.
- When the investor withdraws from the Pool by returning $x$ fT tokens, he
receives $x * f_w / f_d$ where $f_d$ is the Pool's factor $f$ recorded at the time with the investor's deposit, and $f_w$ is the curren Pool's factor $f$ (at the time of withdrawal).

The above calculations stimulate initial investors, but not unfairly. As there
is more need for fT tokens in circulation, there is more need for minting fT,
but also more usage of fT for all investors to enjoy.

To effectively implement the above calculations, $f$ is the ratio of
the amount of deposited T and the total fT minted, calculated only
outside of a flash loan. 
- The withdrawals are always
in-full, paying out $f * x$ of T for the surrendered $x$ amount of fT.
- Upon deposit of $x$ of T, the depositor receives $x/f$ of fT.

Solana's low network fees allow for change of the deposited amount by simply
withdrawing the entire holdings and then depositing the desired amount. This
approach allows for cleaner code and less mistakes. In the future this may be optimized.

So far only the yield obtained from the FLUF Protocol is explained. But there is more: As investors deposit tokens T into the FLUF Protocol
they receive tokens fT, which have
actual collateralized value. As such they can be used in other DeFi protocols
for additional yield, for example to borrow other assets against fT, or
even to borrow more T against fT and re-engage the borrowed T in another round
of investment into the FLUF Protocol. Moreover, borrowing T against fT should
not require much overcollateralization, as fT is an appreciating asset relative
to T and there is no Delta Risk.

Here are some important questions and answers, which clarify the economics
of the Pool's operation:
- Q: Why do we need tokens fT outside of Flash Loans?
- A: Because they have actual collateralized value, for other protocols to
trade and the FLUF Protocol to receive fees.

--
- Q: Why can then fT be minted in any amount inside Flash Loans? Isn't that frivolous?
- A: It is not frivolous because whatever is minted has to be burned at the
end in the same amount. No one profits and no one loses. This can be viewed
as stimulus in order to perform arbitrage, liquidate delinquent derivative
positions, liquidate delinquent loans etc.

## Implementation and Integration Instructions

## Known Issues

If the Flash Loan is needed to borrow T instead of fT, the standard pattern would be to
borrow fT, convert it to T up to a maximum amount that exists in the pool and then,
after usage, deposit back the borrowed T in addition to the appropriate amount for fees and
repay the loan and the fees. However, since an arbitrary amount of fT can be borrowed,
the caller can convert the entire pool from fT to T. Leaving nothing in it, this loses
the information about the prior ratio of fT and T. This would allow the borrower to
steal funds. To remedy this issue, the FLUF Protocol would have to keep track of minting
fT without collateral in T, which is an indication that this is happening inside of a
Flash Loan. In such case, the prior ratio $f$ of T and fT should be recorded, and enforced.

Since Solana only allows for 4 levels of CPI calls, the FLUF Protocol takes one level out
of this capability. To remedy this, the calling sequence at the lowest level should be achieved
via a sequence of Solana Instructions instead of one instruction that calls the FLUF Protocol
```lendAndCall``` entry point. However, the sequence of Instructions should be enforced and
the repayment balances recorded (in a PDA).

## Conclusion

